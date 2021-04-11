pub use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub use pulse::{
	context::{subscribe::Facility, Context as PAContext},
	mainloop::{api::Mainloop as MainloopTrait, threaded::Mainloop},
	stream::Stream,
};
pub use tokio::sync::mpsc;

pub use super::{errors::PAError, monitor::Monitors, PAInternal, SPEC};
pub use crate::{
	entry::{EntryIdentifier, EntryType},
	models::{EntryUpdate, PulseAudioAction},
	prelude::*,
};

pub static LOGGING_MODULE: &str = "PAInterface";

impl From<Facility> for EntryType {
	fn from(fac: Facility) -> Self {
		match fac {
			Facility::Sink => EntryType::Sink,
			Facility::Source => EntryType::Source,
			Facility::SinkInput => EntryType::SinkInput,
			Facility::SourceOutput => EntryType::SourceOutput,
			Facility::Card => EntryType::Card,
			_ => EntryType::Sink,
		}
	}
}
