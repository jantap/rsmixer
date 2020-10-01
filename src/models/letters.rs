use crate::{
    events::{MAIN_MESSAGE, PA_MESSAGE},
    entry::{Entry, EntryIdentifier},
    messages,
    models::PageType,
    Message,
};

use std::collections::HashMap;

use pulse::volume::ChannelVolumes;

messages!(Letter,
    ExitSignal => 0,

    Redraw => MAIN_MESSAGE,
    EntryRemoved(EntryIdentifier) => MAIN_MESSAGE,
    EntryUpdate(EntryIdentifier, Box<Entry>) => MAIN_MESSAGE,
    PeakVolumeUpdate(EntryIdentifier, f32) => MAIN_MESSAGE,
    RequstChangeVolume(i16) => MAIN_MESSAGE,
    MoveUp(u16) => MAIN_MESSAGE,
    MoveDown(u16) => MAIN_MESSAGE,
    ChangePage(PageType) => MAIN_MESSAGE,
    RequestMute => MAIN_MESSAGE,
    OpenContextMenu => MAIN_MESSAGE,
    CloseContextMenu => MAIN_MESSAGE,
    CyclePages(i8) => MAIN_MESSAGE,
    ShowHelp => MAIN_MESSAGE,
    Hide => MAIN_MESSAGE,

    MuteEntry(EntryIdentifier, bool) => PA_MESSAGE,
    MoveEntryToParent(EntryIdentifier, EntryIdentifier) => PA_MESSAGE,
    ChangeCardProfile(EntryIdentifier, String) => PA_MESSAGE,
    SetVolume(EntryIdentifier, ChannelVolumes) => PA_MESSAGE,
    CreateMonitors(HashMap<EntryIdentifier, Option<u32>>) => PA_MESSAGE,
    SetSuspend(EntryIdentifier, bool) => PA_MESSAGE,
    KillEntry(EntryIdentifier) => PA_MESSAGE,
);
