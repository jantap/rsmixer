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
    unbounded_channel, UnboundedReceiver as Receiver, UnboundedSender as Sender,
};

pub fn new() -> (Ctx, Worker) {
    let (sx, rx) = unbounded_channel();

    (sx.clone().into(), Worker::new(sx, rx))
}
