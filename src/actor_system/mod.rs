mod actor;
mod actor_entry;
mod context;
mod error;
mod messages;
pub mod prelude;
mod worker;

pub use actor::{Actor, ActorStatus, ActorType, ContinousActor, EventfulActor};
pub use context::Ctx;
pub use error::Error;
pub use messages::BoxedMessage;
pub use worker::Worker;

use tokio::sync::mpsc::{
    unbounded_channel as channel, UnboundedReceiver as Receiver, UnboundedSender as Sender,
};

static LOGGING_MODULE: &'static str = "ActorSystem";

pub fn new() -> (Ctx, Worker) {
    let (sx, rx) = channel();

    (sx.clone().into(), Worker::new(sx, rx))
}
