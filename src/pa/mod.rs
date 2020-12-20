mod async_loop;
mod callbacks;
mod common;
mod monitor;
mod pa_actions;
mod sync_loop;

pub use async_loop::start_async;
pub use sync_loop::start;

use common::*;

use lazy_static::lazy_static;

#[derive(Debug)]
pub enum PAInternal {
    Tick,
    Command(Box<Letter>),
    AskInfo(EntryIdentifier),
}

lazy_static! {
    pub static ref SPEC: pulse::sample::Spec = pulse::sample::Spec {
        format: pulse::sample::SAMPLE_FLOAT32,
        channels: 1,
        rate: 15,
    };
}
