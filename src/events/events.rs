use super::{PA_MESSAGE, UI_MESSAGE};

use crate::{
    entry::{Entry, EntryIdentifier},
    messages,
    ui::PageType,
    Message,
};

use std::collections::HashMap;

use pulse::volume::ChannelVolumes;

messages!(Letter,
    ExitSignal => 0,

    Redraw => UI_MESSAGE,
    EntryRemoved(EntryIdentifier) => UI_MESSAGE,
    EntryUpdate(EntryIdentifier, Box<Entry>) => UI_MESSAGE,
    PeakVolumeUpdate(EntryIdentifier, f32) => UI_MESSAGE,
    RequstChangeVolume(i16) => UI_MESSAGE,
    MoveUp(u16) => UI_MESSAGE,
    MoveDown(u16) => UI_MESSAGE,
    ChangePage(PageType) => UI_MESSAGE,
    RequestMute => UI_MESSAGE,
    OpenContextMenu => UI_MESSAGE,
    CyclePages(i8) => UI_MESSAGE,

    MuteEntry(EntryIdentifier, bool) => PA_MESSAGE,
    MoveEntryToParent(EntryIdentifier, EntryIdentifier) => PA_MESSAGE,
    ChangeCardProfile(EntryIdentifier, String) => PA_MESSAGE,
    SetVolume(EntryIdentifier, ChannelVolumes) => PA_MESSAGE,
    CreateMonitors(HashMap<EntryIdentifier, Option<u32>>) => PA_MESSAGE,
    SetSuspend(EntryIdentifier, bool) => PA_MESSAGE,
    KillEntry(EntryIdentifier) => PA_MESSAGE,
);
