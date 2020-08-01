use crate::{bishopify, BishopMessage};
use crate::{
    entry::{Entry, EntryIdentifier},
    ui::PageType,
};
use pulse::volume::ChannelVolumes;
use std::collections::HashMap;

pub static CHANNEL_CAPACITY: usize = 32;

pub static UI_MESSAGE: u32 = 1;
pub static PA_MESSAGE: u32 = 2;

type MonSrc = Option<u32>;
type M = HashMap<EntryIdentifier, MonSrc>;
bishopify!(Letter,
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
