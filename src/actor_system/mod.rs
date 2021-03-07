mod actor;
mod actor_entry;
mod context;
mod error;
mod messages;
pub mod prelude;
mod worker;

pub use actor::{Actor, ActorStatus, BoxedActor};
pub use context::Ctx;
pub use error::Error;
pub use messages::BoxedMessage;
pub use worker::Worker;

use tokio::sync::broadcast::{channel, Receiver, Sender};

pub fn new() -> (Ctx, Worker) {
    let (sx, rx) = channel(128);

    (sx.clone().into(), Worker::new(sx, rx))
}
