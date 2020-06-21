use crate::{bishopify, BishopMessage};
use crate::{ui::PageType, EntryIdentifier, EntryType};
use pulse::volume::ChannelVolumes;

pub static CHANNEL_CAPACITY: usize = 32;

pub static UI_MESSAGE: u32 = 1;
pub static PA_MESSAGE: u32 = 2;

bishopify!(Letter,
    ExitSignal => 0,

    Redraw => UI_MESSAGE,
    EntryRemoved(EntryIdentifier) => UI_MESSAGE,
    EntryUpdate(EntryIdentifier) => UI_MESSAGE,
    PeakVolumeUpdate(EntryIdentifier, f32) => UI_MESSAGE,
    RequstChangeVolume(i16) => UI_MESSAGE,
    MoveUp(u16) => UI_MESSAGE,
    MoveDown(u16) => UI_MESSAGE,
    ChangePage(PageType) => UI_MESSAGE,
    RequestMute => UI_MESSAGE,

    MuteEntry(EntryIdentifier, bool) => PA_MESSAGE,
    SetVolume(EntryIdentifier, ChannelVolumes) => PA_MESSAGE,
);
