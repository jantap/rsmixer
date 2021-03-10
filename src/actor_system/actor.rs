use super::{
    actor_entry::ActorEntry,
    context::Ctx,
    messages::{BoxedMessage, Shutdown, SystemMessage},
    prelude::MessageReceiver,
    Sender,
};

use std::sync::Arc;

use tokio::task::{self, JoinHandle};

use async_trait::async_trait;

pub type BoxedEventfulActor = Box<dyn EventfulActor + Send + Sync>;
pub type BoxedContinousActor = Box<dyn ContinousActor + Send + Sync>;

pub type ActorFactory = &'static (dyn Fn() -> Actor + Send + Sync);

#[derive(PartialEq)]
pub enum ActorStatus {
    Off,
    Ready,
    Handling,
    NotRegistered,
    Starting,
    Stopping,
}

#[async_trait]
pub trait EventfulActor {
    async fn start(&mut self, ctx: Ctx) -> Result<(), anyhow::Error>;
    async fn stop(&mut self);
    async fn handle_message(&mut self, ctx: Ctx, msg: BoxedMessage) -> Result<(), anyhow::Error>;
}

#[async_trait]
pub trait ContinousActor {
    async fn start(&mut self, ctx: Ctx, events_rx: MessageReceiver) -> Result<(), anyhow::Error>;
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

#[derive(PartialEq)]
pub enum ActorType {
    Eventful,
    Continous,
}

pub fn spawn_actor(sx: Sender<Arc<SystemMessage>>, id: &'static str, mut actor: Actor) {
    task::spawn(async move {
        let (res, ssx) = {
            match &mut actor {
                Actor::Eventful(actor) => (actor.start(sx.clone().into()).await, None),
                Actor::Continous(actor) => {
                    let (ssx, rrx) = tokio::sync::mpsc::unbounded_channel();
                    (actor.start(sx.clone().into(), rrx).await, Some(ssx))
                }
            }
        };
        let r = sx.send(Arc::new(SystemMessage::ActorUpdate(
            id,
            ActorEntry::new(
                Some(actor),
                if res.is_ok() {
                    ActorStatus::Ready
                } else {
                    ActorStatus::Off
                },
                ssx,
            ),
        )));

        match r {
            Err(_) => Err(()),
            Ok(_) => Ok(()),
        }
        .expect("System might not be properly initialized");
    });
}

pub async fn stop_actor(entry: &mut ActorEntry) {
    if let Some(actor) = &mut entry.actor {
        {
            let mut status = entry.status.write().await;
            *status = ActorStatus::Stopping;
        }

        {
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
    }
}
