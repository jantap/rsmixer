use std::{pin::Pin, sync::Arc};

use async_trait::async_trait;
use futures::Future;
use tokio::{sync::RwLock, task};

use super::{
    actor_entry::ActorEntry,
    channel,
    context::Ctx,
    messages::{BoxedMessage, Shutdown, SystemMessage},
    prelude::MessageReceiver,
    retry_strategy::{RetryStrategy, Strategy},
    Receiver, Sender, LOGGING_MODULE,
};
use crate::prelude::*;

pub type BoxedEventfulActor = Box<dyn EventfulActor + Send + Sync>;
pub type BoxedContinousActor = Box<dyn ContinousActor + Send + Sync>;

pub type ActorFactory = &'static (dyn Fn() -> Actor + Send + Sync);

#[derive(Clone)]
pub struct ActorStatusLocked(Arc<RwLock<ActorStatus>>);

impl ActorStatusLocked {
    pub fn new(status: ActorStatus) -> Self {
        Self {
            0: Arc::new(RwLock::new(status)),
        }
    }
    pub async fn set(&self, status: ActorStatus) {
        let mut stat = self.0.write().await;
        *stat = status;
    }
    pub async fn get(&self) -> ActorStatus {
        let stat = self.0.read().await;
        *stat
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ActorStatus {
    Off,
    Ready,
    Failed,
    NotRegistered,
    Starting,
    Stopping,
}

#[async_trait]
pub trait EventfulActor {
    async fn start(&mut self, ctx: Ctx) -> Result<()>;
    async fn stop(&mut self);
    async fn handle_message(&mut self, ctx: Ctx, msg: BoxedMessage) -> Result<()>;
}

pub type BoxedResultFuture = Pin<Box<dyn Future<Output = Result<()>> + Send + Sync>>;

#[async_trait]
pub trait ContinousActor {
    async fn start(&mut self, ctx: Ctx) -> Result<()>;
    fn run(&mut self, ctx: Ctx, events_rx: MessageReceiver) -> BoxedResultFuture;
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
}

#[derive(PartialEq, Copy, Clone)]
pub enum ActorType {
    Eventful,
    Continous,
}

pub struct ActorBlueprint {
    id: &'static str,
    factory: ActorFactory,
    pub retry_strategy: RetryStrategy,
}

impl ActorBlueprint {
    pub fn new(id: &'static str, factory: ActorFactory) -> Self {
        Self {
            id,
            factory,
            retry_strategy: RetryStrategy::default(),
        }
    }

    pub fn on_panic<F: Strategy<usize> + 'static>(mut self, f: F) -> Self {
        self.retry_strategy.on_panic = Box::new(f);
        self
    }

    pub fn on_error<F: Strategy<(usize, Result<()>)> + 'static>(mut self, f: F) -> Self {
        self.retry_strategy.on_error = Box::new(f);
        self
    }

    pub fn start(self, ctx: &mut Ctx) {
        ctx.actor_with_retry_strategy(self.id, self.factory, self.retry_strategy);
    }
}

fn eventful_actor_msg_handler(
    actor_id: &'static str,
    mut rx: Receiver<BoxedMessage>,
    ctx: Ctx,
    actor: Arc<RwLock<Actor>>,
    status: ActorStatusLocked,
) {
    task::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if msg.is::<Shutdown>() {
                break;
            }

            let actor = Arc::clone(&actor);
            let ctx_copy = ctx.clone();

            let result = task::spawn(async move {
                let mut actor = actor.write().await;
                match &mut *actor {
                    Actor::Eventful(actor) => actor.handle_message(ctx_copy, msg).await,
                    _ => Ok(()),
                }
            })
            .await;

            match result {
                Ok(Err(err)) => {
                    warn!("Actor '{}' returned Err while handling message", actor_id);
                    status.set(ActorStatus::Failed).await;

                    ctx.actor_returned_err(actor_id, Err(err));

                    return;
                }
                Err(_) => {
                    warn!("Actor '{}' panicked while handling message", actor_id);
                    status.set(ActorStatus::Failed).await;

                    ctx.actor_panicked(actor_id);

                    return;
                }
                _ => {}
            }
        }
    });
}

pub fn spawn_actor(sx: Sender<Arc<SystemMessage>>, id: &'static str, mut actor: Actor) {
    task::spawn(async move {
        info!("Spawning actor '{}'", id);
        let (ssx, rrx) = channel::<BoxedMessage>();

        let res = {
            match &mut actor {
                Actor::Eventful(actor) => actor.start(sx.clone().into()).await,
                Actor::Continous(actor) => actor.start(sx.clone().into()).await,
            }
        };

        let actor_entry = ActorEntry::new(
            Some(actor),
            if res.is_ok() {
                ActorStatus::Ready
            } else {
                ActorStatus::Off
            },
            Some(ssx),
        );

        if let Some(ActorType::Eventful) = actor_entry.actor_type {
            if let Some(actor) = &actor_entry.actor {
                eventful_actor_msg_handler(
                    id,
                    rrx,
                    sx.clone().into(),
                    Arc::clone(&actor),
                    actor_entry.status.clone(),
                );
            }
        } else if let Some(actor) = &actor_entry.actor {
            let task = {
                let mut actor = actor.write().await;
                if let Actor::Continous(actor) = &mut *actor {
                    actor.run(sx.clone().into(), rrx)
                } else {
                    Box::pin(async { Ok(()) })
                }
            };

            let status = actor_entry.status.clone();
            let ctx: Ctx = sx.clone().into();

            let handle = task::spawn(async move {
                let result = task::spawn(task).await;

                match result {
                    Ok(Err(err)) => {
                        warn!("Actor '{}' returned Err while handling message", id);
                        status.set(ActorStatus::Failed).await;

                        ctx.actor_returned_err(id, Err(err));
                    }
                    Err(_) => {
                        warn!("Actor '{}' panicked while handling message", id);
                        status.set(ActorStatus::Failed).await;

                        ctx.actor_panicked(id);
                    }
                    _ => {}
                }
            });

            let _ = sx.send(Arc::new(SystemMessage::UserTask(handle)));
        }

        let r = sx.send(Arc::new(SystemMessage::ActorUpdate(id, actor_entry)));

        info!("Started actor '{}'", id);

        match r {
            Err(_) => Err(()),
            Ok(_) => Ok(()),
        }
        .expect("System might not be properly initialized");
    });
}

pub async fn stop_actor(id: &'static str, entry: &mut ActorEntry) {
    info!("Stopping actor '{}'", id);
    if let Some(actor) = &mut entry.actor {
        entry.status.set(ActorStatus::Stopping).await;
        let mut actor = actor.write().await;

        match &mut *actor {
            Actor::Eventful(actor) => {
                actor.stop().await;
            }
            Actor::Continous(actor) => {
                if let Some(sx) = &entry.event_sx {
                    let _ = sx.send(Box::new(Shutdown {}));
                }

                actor.stop().await;
            }
        };
    }
    info!("Stopped actor '{}'", id);
}
