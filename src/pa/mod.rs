mod callbacks;
pub mod common;
mod errors;
mod monitor;
mod pa_actions;
mod pa_interface;

use common::*;
use lazy_static::lazy_static;
pub use pa_interface::start;

#[derive(Debug)]
pub enum PAInternal {
	Tick,
	Command(Box<PulseAudioAction>),
	AskInfo(EntryIdentifier),
}

lazy_static! {
	pub static ref SPEC: pulse::sample::Spec = pulse::sample::Spec {
		format: pulse::sample::Format::FLOAT32NE,
		channels: 1,
		rate: 1024,
	};
}
