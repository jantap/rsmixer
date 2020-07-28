mod async_loop;
mod callbacks;
mod common;
mod monitor;
mod pa_actions;
mod sync_loop;

use common::*;
use lazy_static::lazy_static;
use state::Storage;
use tokio::sync::mpsc;

pub use async_loop::start_async;
pub use sync_loop::start;

pub enum PAInternal {
    Tick,
    Command(Letter),
    AskInfo(EntryIdentifier),
}

lazy_static! {
    pub static ref INFO_SX: Storage<mpsc::UnboundedSender<EntryIdentifier>> = Storage::new();
    pub static ref SPEC: pulse::sample::Spec = pulse::sample::Spec {
        format: pulse::sample::SAMPLE_FLOAT32,
        channels: 1,
        rate: 15,
    };
}
