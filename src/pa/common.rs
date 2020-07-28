pub use super::{monitor::Monitors, PAInternal, INFO_SX, SPEC};
pub use crate::entry::{EntryIdentifier, EntryType};
pub use crate::{Letter, RSError, DISPATCH};
pub use log::{debug, error, info};
pub use pulse::stream::Stream;
pub use pulse::{
    context::{subscribe::Facility, Context},
    mainloop::{
        api::Mainloop as MainloopTrait, //Needs to be in scope
        threaded::Mainloop,
    },
};
pub use std::cell::RefCell;
pub use std::collections::HashMap;
pub use std::rc::Rc;

impl From<Facility> for EntryType {
    fn from(fac: Facility) -> Self {
        match fac {
            Facility::Sink => EntryType::Sink,
            Facility::Source => EntryType::Source,
            Facility::SinkInput => EntryType::SinkInput,
            Facility::SourceOutput => EntryType::SourceOutput,
            _ => EntryType::Sink,
        }
    }
}
