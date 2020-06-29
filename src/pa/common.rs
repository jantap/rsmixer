
pub use std::collections::HashMap;
pub use std::cell::RefCell;
pub use std::rc::Rc;
pub use pulse::stream::Stream;
pub use crate::{DISPATCH, RSError, Letter};
pub use crate::entry::{EntryType, EntryIdentifier};
pub use super::{Monitors, PAInternal, INFO_SX, SPEC};
pub use pulse::{
    context::{
        Context,
        subscribe::Facility,
    },
    mainloop::{
        api::Mainloop as MainloopTrait, //Needs to be in scope
        threaded::Mainloop,
    },
};
pub use log::{debug, error,info};

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
