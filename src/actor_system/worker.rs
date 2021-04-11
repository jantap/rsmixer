use std::{collections::HashMap, sync::Arc};

use tokio::task;

use super::{
	actor::ActorItem,
	context::Ctx,
	messages::{BoxedMessage, SystemMessage},
	Receiver, Sender, LOGGING_MODULE,
};
use crate::prelude::*;

pub struct RegisteredActors {
	items: HashMap<&'static str, ActorItem>,
	ctx: Ctx,
}

impl RegisteredActors {
	pub fn new(ctx: Ctx) -> Self {
		Self {
			items: HashMap::new(),
			ctx,
		}
	}

	pub fn register(&mut self, item: ActorItem) {
		self.items.insert(item.id, item);
	}

	pub async fn stop(&mut self, id: &'static str) {
		if let Some(item) = self.items.get_mut(id) {
			item.stop().await;
		}
	}

	pub async fn restart(&mut self, id: &'static str) {
		if let Some(item) = self.items.get_mut(id) {
			item.restart().await;
		}
	}

	pub async fn stop_and_cache_messages(&mut self, id: &'static str) {
		if let Some(item) = self.items.get_mut(id) {
			item.stop_and_cache_messages().await;
		}
	}

	pub async fn start(&mut self, id: &'static str) -> Result<()> {
		if let Some(item) = self.items.get_mut(id) {
			item.start(&self.ctx).await
		} else {
			Err(anyhow::anyhow!("actor is not registered"))
		}
	}

	pub fn send(&mut self, id: &'static str, msg: BoxedMessage) {
		if let Some(item) = self.items.get_mut(id) {
			item.send(msg);
		}
	}

	pub async fn actor_task_finished(&mut self, id: &'static str, result: Option<Result<()>>) {
		if let Some(item) = self.items.get_mut(id) {
			item.actor_task_finished(&self.ctx, result).await;
		}
	}

	pub async fn shutdown(&mut self) {
		for (_, item) in &mut self.items {
			item.shutdown();
		}

		for (_, item) in &mut self.items {
			item.join().await;
		}
	}
}

pub struct Worker {
	actors: RegisteredActors,
	internal_rx: Option<Receiver<Arc<SystemMessage>>>,
}
impl Worker {
	pub fn new(sx: Sender<Arc<SystemMessage>>, rx: Receiver<Arc<SystemMessage>>) -> Self {
		Self {
			actors: RegisteredActors::new(sx.clone().into()),
			internal_rx: Some(rx),
		}
	}

	pub fn start(mut self) -> tokio::task::JoinHandle<Result<()>> {
		let mut i_rx = self.internal_rx.take().unwrap();
		task::spawn(async move {
			debug!("Starting worker task");

			while let Some(msg) = i_rx.recv().await {
				let msg = match Arc::try_unwrap(msg) {
					Ok(x) => x,
					Err(_) => {
						error!("Failed to unwrap Arc<SystemMessage>. Skipping message. This may be very bad news");
						continue;
					}
				};
				match msg {
					SystemMessage::RegisterActor(item) => {
						self.actors.register(item);
					}
					SystemMessage::StartActor(id) => {
						if let Err(e) = self.actors.start(id).await {
							error!("Failed to start actor {}.\n{:#?}", id, e);
						}
					}
					SystemMessage::StopActor(id) => {
						self.actors.stop(id).await;
					}
					SystemMessage::SendMsg(id, m) => {
						self.actors.send(id, m);
					}
					SystemMessage::ActorTaskFinished(id, result) => {
						self.actors.stop_and_cache_messages(id).await;

						self.actors.actor_task_finished(id, result).await;
					}
					SystemMessage::RestartActor(id) => {
						self.actors.restart(id).await;
					}
					SystemMessage::Shutdown => {
						self.actors.shutdown().await;
						break;
					}
				};
			}
			Ok(())
		})
	}
}
