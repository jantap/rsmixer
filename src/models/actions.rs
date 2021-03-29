use std::collections::HashMap;

use crossterm::event::Event;
use pulse::volume::ChannelVolumes;

use crate::{
    entry::{Entry, EntryIdentifier},
    models::PageType,
};

#[derive(Clone, PartialEq, Debug)]
pub enum PAStatus {
    // PulseAudio connection status
    RetryIn(u64),
    ConnectToPulseAudio,
    PulseAudioDisconnected,
}

// redraw the whole screen (called every window resize)
#[derive(Clone, PartialEq, Debug)]
pub struct ResizeScreen {}
impl ResizeScreen {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum EntryUpdate {
    // entry updates
    EntryRemoved(EntryIdentifier),
    EntryUpdate(EntryIdentifier, Box<Entry>),
    PeakVolumeUpdate(EntryIdentifier, f32),
}

#[derive(Clone, PartialEq, Debug)]
pub struct UserInput {
    pub event: Event,
}
impl UserInput {
    pub fn new(event: Event) -> Self {
        Self { event }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum UserAction {
    // move around the UI
    MoveUp(u16),
    MoveDown(u16),
    MoveLeft,
    MoveRight,
    SetSelected(usize),
    ChangePage(PageType),
    // positive - forwards, negative - backwards
    CyclePages(i8),

    // volume changes
    RequestMute(Option<EntryIdentifier>),

    // request volume change where the argument is a
    // number of percentage points it should be changed by
    RequstChangeVolume(i16, Option<EntryIdentifier>),

    // context menus
    OpenContextMenu(Option<EntryIdentifier>),
    CloseContextMenu,
    Confirm,

    ShowHelp,

    Hide(Option<EntryIdentifier>),

    RequestQuit,
}

#[derive(Clone, PartialEq, Debug)]
pub enum PulseAudioAction {
    RequestPulseAudioState,
    MuteEntry(EntryIdentifier, bool),
    MoveEntryToParent(EntryIdentifier, EntryIdentifier),
    ChangeCardProfile(EntryIdentifier, String),
    SetVolume(EntryIdentifier, ChannelVolumes),
    CreateMonitors(HashMap<EntryIdentifier, Option<u32>>),
    SetSuspend(EntryIdentifier, bool),
    KillEntry(EntryIdentifier),
    Shutdown,
}
