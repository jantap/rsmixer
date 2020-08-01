use crate::{messages, Message};
use crate::{
    entry::{Entry, EntryIdentifier},
    ui::PageType,
};
use pulse::volume::ChannelVolumes;
use std::collections::HashMap;

use super::{UI_MESSAGE, PA_MESSAGE};

type MonSrc = Option<u32>;
type M = HashMap<EntryIdentifier, MonSrc>;

messages!(Letter,
    ExitSignal => 0,

    Redraw => UI_MESSAGE,
    EntryRemoved(EntryIdentifier) => UI_MESSAGE,
    EntryUpdate(EntryIdentifier, Entry) => UI_MESSAGE,
    PeakVolumeUpdate(EntryIdentifier, f32) => UI_MESSAGE,
    RequstChangeVolume(i16) => UI_MESSAGE,
    MoveUp(u16) => UI_MESSAGE,
    MoveDown(u16) => UI_MESSAGE,
    ChangePage(PageType) => UI_MESSAGE,
    RequestMute => UI_MESSAGE,
    OpenContextMenu => UI_MESSAGE,

    MuteEntry(EntryIdentifier, bool) => PA_MESSAGE,
    MoveEntryToParent(EntryIdentifier, EntryIdentifier) => PA_MESSAGE,
    ChangeCardProfile(EntryIdentifier, String) => PA_MESSAGE,
    SetVolume(EntryIdentifier, ChannelVolumes) => PA_MESSAGE,
    CreateMonitors(M) => PA_MESSAGE,
    SetSuspend(EntryIdentifier, bool) => PA_MESSAGE,
    KillEntry(EntryIdentifier) => PA_MESSAGE,
);
