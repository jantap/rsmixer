use crate::ui::{widgets::VolumeWidget, Rect};

use super::EntrySpaceLvl;
use crate::entry::EntryType;

#[derive(PartialEq, Clone, Debug)]
pub struct PlayEntry {
    pub peak: f32,
    pub mute: bool,
    pub volume: pulse::volume::ChannelVolumes,
    pub monitor_source: Option<u32>,
    pub sink: Option<u32>,
    pub volume_bar: VolumeWidget,
    pub peak_volume_bar: VolumeWidget,
    pub suspended: bool,
}
impl Eq for PlayEntry {}

#[derive(PartialEq, Clone, Debug)]
pub struct CardProfile {
    pub name: String,
    pub description: String,
    pub available: bool,
}
impl Eq for CardProfile {}

#[derive(PartialEq, Clone, Debug)]
pub struct CardEntry {
    pub profiles: Vec<CardProfile>,
    pub selected_profile: Option<usize>,
}
impl Eq for CardEntry {}

#[derive(PartialEq, Clone, Debug)]
pub struct Entry {
    pub entry_type: EntryType,
    pub index: u32,
    pub name: String,
    pub is_selected: bool,
    pub parent: Option<u32>,
    pub position: EntrySpaceLvl,
    pub play_entry: Option<PlayEntry>,
    pub card_entry: Option<CardEntry>,
}
impl Eq for Entry {}

impl Entry {
    pub fn calc_area(position: EntrySpaceLvl, mut area: Rect) -> Rect {
        let amount = match position {
            EntrySpaceLvl::Card => 1,
            EntrySpaceLvl::Parent => 2,
            EntrySpaceLvl::ParentNoChildren => 2,
            _ => 5,
        };

        area.x += amount;
        area.width -= amount;

        area
    }
}
