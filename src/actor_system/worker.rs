use std::{collections::HashMap, sync::Arc};

use tokio::{
	sync::RwLock,
	task::{self, JoinHandle},
};

use super::{
	actor::{ActorFactory, ActorStatus, ActorType},
	actor_entry::ActorEntry,
	messages::{BoxedMessage, SystemMessage},
	retry_worker::RetryWorker,
	Receiver, Sender, LOGGING_MODULE,
};
use crate::prelude::*;

pub struct Worker {
	entries: HashMap<&'static str, ActorEntry>,
	factories: HashMap<&'static str, ActorFactory>,
	internal_rx: Option<Receiver<Arc<SystemMessage>>>,
	internal_sx: Sender<Arc<SystemMessage>>,
	user_tasks: Vec<JoinHandle<()>>,
	retry_worker: RetryWorker,
}
impl Worker {
	pub fn new(sx: Sender<Arc<SystemMessage>>, rx: Receiver<Arc<SystemMessage>>) -> Self {
		Self {
			entries: HashMap::new(),
			factories: HashMap::new(),
			internal_sx: sx,
			internal_rx: Some(rx),
			user_tasks: Vec::new(),
			retry_worker: RetryWorker::default(),
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
					SystemMessage::ActorRegistered(id, factory, retry_strategy) => {
						self.factories.insert(id, factory);
						self.retry_worker.add_strategy(id, retry_strategy);
					}
					SystemMessage::ActorUpdate(id, mut actor) => {
						if let Some(a) = self.entries.get(id) {
							actor.cached_messages = a.cached_messages.to_owned();
						}
						self.entries.insert(id, actor);
					}
					SystemMessage::SendMsg(id, m) => {
						self.send_to(id, m).await;
					}
					SystemMessage::ActorPanicked(id) => {
						self.retry_worker
							.actor_panic(id, self.internal_sx.clone().into());
					}
					SystemMessage::ActorReturnedErr(id, result) => {
						self.retry_worker.actor_returned_err(
							id,
							result,
							self.internal_sx.clone().into(),
						);
					}
					SystemMessage::UserTask(handle) => {
						self.user_tasks.push(handle);
					}
					SystemMessage::RestartActor(id) => {
						self.restart_actor(id).await;
					}
					SystemMessage::Shutdown => {
						self.shutdown().await;
						break;
					}
				};
			}
			Ok(())
		})
	}

	async fn send_to(&mut self, id: &'static str, msg: BoxedMessage) {
		if let Some(entry) = self.entries.get(id) {
			if let Some(ActorType::Continous) = entry.actor_type {
				if let Some(sx) = &entry.event_sx {
					let _ = sx.send(msg);
					return;
				}
			}
		}

		self.send_to_eventful(id, msg).await;
	}

	async fn send_to_eventful(&mut self, id: &'static str, msg: BoxedMessage) {
		let mut sx = None;
		if let Some(act_entry) = self.entries.get_mut(id) {
			if act_entry.actor.is_some()
				&& act_entry.actor_type == Some(ActorType::Eventful)
				&& act_entry.status.get().await == ActorStatus::Ready
			{
				if let Some(s) = &act_entry.event_sx {
					sx = Some(s.clone());
				}
			}
		}
		if let Some(sx) = sx {
			let cached = self.cached_messages_for(id);
			let mut cached = cached.write().await;
			let mut to_send = vec![msg];

			while let Some(c) = cached.pop() {
				to_send.push(c);
			}

			while let Some(c) = to_send.pop() {
				let _ = sx.send(c);
			}
			return;
		}

		let cached = self.cached_messages_for(id);
		let mut cached = cached.write().await;

		cached.push(msg);
	}

	async fn shutdown(&mut self) {
		info!("Shutting down");
		self.retry_worker.abort_arbitrers();

		for entry in self.entries.values_mut() {
			{
				let mut cached = entry.cached_messages.write().await;
				cached.clear();
			}
		}

		for (id, entry) in &mut self.entries {
			super::actor::stop_actor(id, entry).await;
		}

		for task in &mut self.user_tasks {
			let _ = task.await;
		}

		info!("Finished");
	}

	async fn restart_actor(&mut self, id: &'static str) {
		info!("Restarting actor {}", id);

		if let Some(entry) = self.entries.get_mut(id) {
			super::actor::stop_actor(id, entry).await;
		}

		if let Some(factory) = self.factories.get(id) {
			let actor = (factory)();
			super::actor::spawn_actor(self.internal_sx.clone(), id, actor);
		}
	}

	fn cached_messages_for(&mut self, id: &'static str) -> Arc<RwLock<Vec<BoxedMessage>>> {
		let j = self.entries.get(id).is_some();

		if !j {
			self.entries
				.insert(id, ActorEntry::new(None, ActorStatus::NotRegistered, None));
		}

		Arc::clone(&self.entries.get(id).unwrap().cached_messages)
	}
}
