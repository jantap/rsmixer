pub use async_trait::async_trait;

pub use super::{
    actor::{Actor, ActorStatus, ContinousActor, EventfulActor},
    context::Ctx,
    messages::{Shutdown, BoxedMessage},
    worker::Worker,
};

pub type MessageReceiver = tokio::sync::mpsc::UnboundedReceiver<BoxedMessage>;
pub type MessageSender = tokio::sync::mpsc::UnboundedSender<BoxedMessage>;
