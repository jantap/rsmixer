use super::{
    actor_entry::ActorEntry,
    context::Ctx,
    messages::{BoxedMessage, SystemMessage},
    Sender,
};

use std::sync::Arc;

use tokio::task;

use async_trait::async_trait;

pub type BoxedActor = Box<dyn Actor + Send + Sync>;

pub type ActorFactory = &'static (dyn Fn() -> BoxedActor + Send + Sync);

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
pub trait Actor {
    async fn start(&mut self, ctx: Ctx) -> Result<(), anyhow::Error>;
    async fn stop(&mut self);
    async fn handle_message(&mut self, ctx: Ctx, msg: BoxedMessage) -> Result<(), anyhow::Error>;
}

pub fn spawn_actor(sx: Sender<Arc<SystemMessage>>, id: &'static str, mut actor: BoxedActor) {
    task::spawn(async move {
        let res = actor.start(sx.clone().into()).await;
        let r = sx.send(Arc::new(SystemMessage::ActorUpdate(
            id,
            ActorEntry::new(
                Some(actor),
                if res.is_ok() {
                    ActorStatus::Ready
                } else {
                    ActorStatus::Off
                },
            ),
        )));

        match r {
            Err(_) => Err(()),
            Ok(_) => Ok(()),
        }
        .expect("System might not be properly initialized");
    });
}
