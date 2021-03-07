use crate::{
    entry::{Entry, EntryIdentifier},
    models::PageType,
};

use std::collections::HashMap;

use pulse::volume::ChannelVolumes;

use crossterm::event::Event;

#[derive(Clone, PartialEq, Debug)]
pub enum Action {
    ExitSignal,

    // redraw the whole screen (called every window resize)
    Redraw,

    // entry updates
    EntryRemoved(EntryIdentifier),
    EntryUpdate(EntryIdentifier, Box<Entry>),
    PeakVolumeUpdate(EntryIdentifier, f32),

    // move around the UI
    MoveUp(u16),
    MoveDown(u16),
    MoveLeft,
    MoveRight,
    ChangePage(PageType),
    // positive - forwards, negative - backwards
    CyclePages(i8),

    // volume changes
    RequestMute(Option<EntryIdentifier>),
    InputVolumeValue,
    // request volume change where the argument is a
    // number of percentage points it should be changed by
    RequstChangeVolume(i16, Option<EntryIdentifier>),

    // context menus
    OpenContextMenu(Option<EntryIdentifier>),
    CloseContextMenu,
    Confirm,

    ShowHelp,

    Hide(Option<EntryIdentifier>),

    // PulseAudio connection status
    RetryIn(u64),
    ConnectToPulseAudio,
    PulseAudioDisconnected,

    UserInput(Event),

    MuteEntry(EntryIdentifier, bool),
    MoveEntryToParent(EntryIdentifier, EntryIdentifier),
    ChangeCardProfile(EntryIdentifier, String),
    SetVolume(EntryIdentifier, ChannelVolumes),
    CreateMonitors(HashMap<EntryIdentifier, Option<u32>>),
    SetSuspend(EntryIdentifier, bool),
    KillEntry(EntryIdentifier),
    PulseAudioDisconnected2,
}
