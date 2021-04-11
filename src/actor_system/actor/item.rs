use futures::future::{AbortHandle, Abortable};
use tokio::task;
use tokio_stream::StreamExt;

use super::{
	super::{
		context::Ctx,
		messages::{BoxedMessage, Shutdown},
		retry_strategy::{RetryStrategy, Strategy},
	},
	ActorFactory, ActorInstance, ActorStatus, LockedActorStatus, MessageQueue,
};
use crate::prelude::*;

pub struct ActorItem {
	pub id: &'static str,
	factory: ActorFactory,
	status: LockedActorStatus,
	instance: Option<ActorInstance>,
	message_queue: MessageQueue,
	retry_strategy: RetryStrategy,
	retry_arbitrer: Option<AbortHandle>,
}

impl ActorItem {
	pub fn new(id: &'static str, factory: ActorFactory) -> Self {
		Self {
			id,
			factory,
			status: LockedActorStatus::new(ActorStatus::Off),
			instance: None,
			message_queue: MessageQueue::default(),
			retry_strategy: RetryStrategy::default(),
			retry_arbitrer: None,
		}
	}

	pub fn register_and_start(self, ctx: &mut Ctx) {
		let id = self.id;
		ctx.register_actor(self);
		ctx.start_actor(id);
	}

	pub fn on_error<R: Strategy<(usize, Result<()>)> + 'static>(mut self, r: R) -> Self {
		self.retry_strategy.on_error = Box::new(r);
		self
	}

	pub fn on_panic<R: Strategy<usize> + 'static>(mut self, r: R) -> Self {
		self.retry_strategy.on_panic = Box::new(r);
		self
	}

	pub fn send(&mut self, msg: BoxedMessage) {
		self.message_queue.push(msg);

		if let Some(instance) = &self.instance {
			self.message_queue.send(&instance.event_channel);
		}
	}

	pub async fn stop(&mut self) {
		self.status.set(ActorStatus::Stopping).await;
		if let Some(instance) = &mut self.instance {
			let _ = instance.event_channel.send(Box::new(Shutdown {}));
		}
	}

	pub async fn restart(&mut self) {
		self.status.set(ActorStatus::Restarting).await;
		if let Some(instance) = &mut self.instance {
			let _ = instance.event_channel.send(Box::new(Shutdown {}));
		}
	}

	pub async fn actor_task_finished(&mut self, ctx: &Ctx, result: Option<Result<()>>) {
		match self.status.get().await {
			ActorStatus::Ready => {
				let strategy = match result {
					Some(Ok(_)) => {
						self.status.set(ActorStatus::Off).await;
						return;
					}
					Some(Err(err)) => {
						let strategy = &mut self.retry_strategy;
						strategy.retry_count += 1;

						(strategy.on_error)((strategy.retry_count - 1, Err(err)))
					}
					None => {
						let strategy = &mut self.retry_strategy;
						strategy.retry_count += 1;

						(strategy.on_panic)(strategy.retry_count - 1)
					}
				};

				let id = self.id;
				let ctx = ctx.clone();
				let status = self.status.clone();
				self.status.set(ActorStatus::ArbiterRunning).await;

				let (handle, registration) = AbortHandle::new_pair();
				self.retry_arbitrer = Some(handle);

				task::spawn(Abortable::new(
					async move {
						if strategy.await {
							ctx.start_actor(id);
						} else {
							status.set(ActorStatus::Off).await;
						}
					},
					registration,
				));
			}
			ActorStatus::Restarting => {
				let _ = self.start(ctx).await;
			}
			ActorStatus::Stopping => {
				self.status.set(ActorStatus::Off).await;
			}
			_ => {}
		}
	}

	pub async fn stop_and_cache_messages(&mut self) {
		if let Some(instance) = &mut self.instance {
			instance.stop_actor().await;

			let mut rx = instance.get_receiver().await;
			rx.close();

			while let Some(msg) = rx.next().await {
				self.message_queue.push(msg);
			}
		}

		self.instance = None;
	}

	pub fn clean_up_retry_arbitrer(&mut self) {
		if let Some(handle) = &mut self.retry_arbitrer {
			handle.abort();
		}
		self.retry_arbitrer = None;
	}

	pub fn shutdown(&mut self) {
		if let Some(instance) = &mut self.instance {
			let _ = instance.event_channel.send(Box::new(Shutdown {}));
		}

		if let Some(handle) = &mut self.retry_arbitrer {
			handle.abort();
		}
	}

	pub async fn join(&mut self) {
		if let Some(instance) = &mut self.instance {
			instance.join().await;
		}
	}

	pub async fn start(&mut self, ctx: &Ctx) -> Result<()> {
		if !self.status.is_off().await {
			return Err(anyhow::anyhow!("actor is not stopped"));
		}

		self.clean_up_retry_arbitrer();

		let mut actor = (self.factory)();

		actor.start(ctx.clone()).await;

		self.status.set(ActorStatus::Ready).await;

		let actor_type = actor.actor_type();

		self.instance = Some(ActorInstance::new(actor));

		let instance = self.instance.as_mut().unwrap();
		instance
			.start_actor_task(self.id, actor_type, ctx.clone())
			.await;

		self.message_queue.send(&instance.event_channel);

		Ok(())
	}
}
