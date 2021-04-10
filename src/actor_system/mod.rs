mod actor;
mod context;
mod messages;
pub mod prelude;
mod retry_strategy;
mod worker;

pub use actor::{Actor, ActorStatus, ActorType, ContinousActor, EventfulActor};
pub use context::Ctx;
pub use messages::BoxedMessage;
pub use retry_strategy::StrategyClosure;
use tokio::sync::mpsc::{
	unbounded_channel as channel, UnboundedReceiver as Receiver, UnboundedSender as Sender,
};
pub use worker::Worker;

static LOGGING_MODULE: &str = "ActorSystem";

pub fn new() -> (Ctx, Worker) {
	let (sx, rx) = channel();

	(sx.clone().into(), Worker::new(sx, rx))
}
