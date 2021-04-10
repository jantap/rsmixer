use std::sync::Arc;

use tokio::{
	sync::{RwLock, RwLockWriteGuard},
	task::{self, JoinHandle},
};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};

use super::super::{
	actor::{Actor, ActorType, BoxedResultFuture, LockedActor},
	channel,
	context::Ctx,
	messages::{BoxedMessage, Shutdown},
	prelude::LockedReceiver,
	Sender, LOGGING_MODULE,
};
use crate::prelude::*;

pub struct ActorInstance {
	actor: LockedActor,
	pub event_channel: Sender<BoxedMessage>,
	events_rx: LockedReceiver,
	task: Option<JoinHandle<Result<()>>>,
}

impl ActorInstance {
	pub fn new(actor: Actor) -> Self {
		let (sx, rx) = channel();
		let rx = Arc::new(RwLock::new(UnboundedReceiverStream::new(rx)));

		Self {
			actor: LockedActor::new(actor),
			event_channel: sx,
			events_rx: rx,
			task: None,
		}
	}

	pub async fn stop_actor(&mut self) {
		let mut actor = self.actor.write().await;
		actor.stop().await;
	}

	pub async fn get_receiver<'a>(
		&'a mut self,
	) -> RwLockWriteGuard<'a, UnboundedReceiverStream<BoxedMessage>> {
		self.events_rx.write().await
	}

	pub async fn start_actor_task(&mut self, id: &'static str, actor_type: ActorType, ctx: Ctx) {
		let event_loop = match actor_type {
			ActorType::Continous => {
				let mut actor = self.actor.write().await;
				let actor = actor.as_continous().unwrap();
				actor.run(ctx.clone(), Arc::clone(&self.events_rx))
			}
			ActorType::Eventful => {
				let actor = self.actor.clone();
				generate_eventful_actor_loop(actor, Arc::clone(&self.events_rx), ctx.clone())
			}
		};

		let task_future = generate_actor_result_handler(id, ctx.clone(), event_loop);

		let task = task::spawn(task_future);

		self.task = Some(task);
	}

	pub async fn join(&mut self) {
		if let Some(handle) = &mut self.task {
			let _  = handle.await;
		}

		let mut actor = self.actor.write().await;
		actor.stop().await;
	}
}

fn generate_actor_result_handler(
	id: &'static str,
	ctx: Ctx,
	f: BoxedResultFuture,
) -> BoxedResultFuture {
	Box::pin(async move {
		let result = task::spawn(f).await;

		match result {
			Ok(res) => {
				if res.is_err() {
					warn!("Actor '{}' returned Err while handling message", id);
				}
				ctx.actor_returned(id, res);
			}
			Err(_) => {
				warn!("Actor '{}' panicked while handling message", id);
				ctx.actor_panicked(id);
			}
		}

		Ok(())
	})
}

fn generate_eventful_actor_loop(
	actor: LockedActor,
	rx: LockedReceiver,
	ctx: Ctx,
) -> BoxedResultFuture {
	Box::pin(async move {
		let mut actor = actor.write().await;
		let mut rx = rx.write().await;
		while let Some(msg) = rx.next().await {
			if msg.is::<Shutdown>() {
				break;
			}

			let result = match &mut *actor {
				Actor::Eventful(actor) => actor.handle_message(ctx.clone(), msg).await,
				_ => Ok(()),
			};

			if result.is_err() {
				return result;
			}
		}
		Ok(())
	})
}
