pub use super::{monitor::Monitors, PAInternal, SPEC};

pub use crate::{
    entry::{EntryIdentifier, EntryType},
    Letter, RSError, DISPATCH,
};

pub use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub use tokio::sync::mpsc;

pub use pulse::stream::Stream;
pub use pulse::{
    context::{subscribe::Facility, Context},
    mainloop::{
        api::Mainloop as MainloopTrait, //Needs to be in scope
        threaded::Mainloop,
    },
};

pub use log::{debug, error, info, warn};

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
