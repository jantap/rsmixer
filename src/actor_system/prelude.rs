pub use super::{
    actor::{Actor, ActorStatus, ContinousActor, EventfulActor},
    context::Ctx,
    messages::{BoxedMessage, Shutdown},
    worker::Worker,
};

pub use async_trait::async_trait;

use super::{Sender, Receiver};

pub type MessageReceiver = Receiver<BoxedMessage>;
pub type MessageSender = Sender<BoxedMessage>;
