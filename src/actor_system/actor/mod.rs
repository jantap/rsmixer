mod actor_type;
mod instance;
mod item;
mod message_queue;
mod registered_actors;
mod status;

use std::{pin::Pin, sync::Arc};

pub use actor_type::ActorType;
use async_trait::async_trait;
use futures::Future;
pub use instance::ActorInstance;
pub use item::ActorItem;
pub use message_queue::MessageQueue;
pub use status::{ActorStatus, LockedActorStatus};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::{context::Ctx, messages::BoxedMessage, prelude::LockedReceiver};
use crate::prelude::*;

pub type BoxedEventfulActor = Box<dyn EventfulActor + Send + Sync>;
pub type BoxedContinousActor = Box<dyn ContinousActor + Send + Sync>;

pub type BoxedResultFuture = Pin<Box<dyn Future<Output = Result<()>> + Send + Sync>>;

pub type ActorFactory = &'static (dyn Fn() -> Actor + Send + Sync);

#[async_trait]
pub trait EventfulActor {
	async fn start(&mut self, ctx: Ctx);
	async fn stop(&mut self);
	fn handle_message<'a>(
		&'a mut self,
		ctx: Ctx,
		msg: BoxedMessage,
	) -> Pin<Box<dyn Future<Output = Result<()>> + Send + Sync + 'a>>;
}

#[async_trait]
pub trait ContinousActor {
	async fn start(&mut self, ctx: Ctx);
	fn run(&mut self, ctx: Ctx, events_rx: LockedReceiver) -> BoxedResultFuture;
	async fn stop(&mut self);
}

pub enum Actor {
	Eventful(BoxedEventfulActor),
	Continous(BoxedContinousActor),
}

impl Actor {
	pub fn actor_type(&self) -> ActorType {
		match self {
			Self::Eventful(_) => ActorType::Eventful,
			Self::Continous(_) => ActorType::Continous,
		}
	}
	pub async fn start(&mut self, ctx: Ctx) {
		match self {
			Self::Eventful(actor) => actor.start(ctx).await,
			Self::Continous(actor) => actor.start(ctx).await,
		}
	}
	pub async fn stop(&mut self) {
		match self {
			Self::Eventful(actor) => actor.stop().await,
			Self::Continous(actor) => actor.stop().await,
		}
	}
	pub fn as_continous(&mut self) -> Option<&mut BoxedContinousActor> {
		match self {
			Self::Continous(a) => Some(a),
			Self::Eventful(_) => None,
		}
	}
	#[allow(dead_code)]
	pub fn as_eventful(&mut self) -> Option<&mut BoxedEventfulActor> {
		match self {
			Self::Eventful(a) => Some(a),
			Self::Continous(_) => None,
		}
	}
}

#[derive(Clone)]
pub struct LockedActor(Arc<RwLock<Actor>>);

impl LockedActor {
	pub fn new(actor: Actor) -> Self {
		Self {
			0: Arc::new(RwLock::new(actor)),
		}
	}
	#[allow(dead_code)]
	pub async fn read(&self) -> RwLockReadGuard<'_, Actor> {
		self.0.read().await
	}
	pub async fn write(&self) -> RwLockWriteGuard<'_, Actor> {
		self.0.write().await
	}
}
