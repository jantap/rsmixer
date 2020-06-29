use crate::ui::{widgets::VolumeWidget, Rect};

use super::EntrySpaceLvl;
use crate::entry::EntryType;

#[derive(PartialEq, Clone, Debug)]
pub struct Entry {
    pub entry_type: EntryType,
    pub index: u32,
    pub name: String,
    pub peak: f32,
    pub mute: bool,
    pub volume: pulse::volume::ChannelVolumes,
    pub parent: Option<u32>,
    pub monitor_source: Option<u32>,
    pub sink: Option<u32>,
    pub is_selected: bool,
    pub position: EntrySpaceLvl,
    pub volume_bar: VolumeWidget,
    pub peak_volume_bar: VolumeWidget,
    pub suspended: bool,
}
impl Eq for Entry {}

impl Entry {
    pub fn calc_area(position: EntrySpaceLvl, mut area: Rect) -> Rect {
        let amount = match position {
            EntrySpaceLvl::Parent => 2,
            EntrySpaceLvl::ParentNoChildren => 2,
            _ => 5,
        };

        area.x += amount;
        area.width -= amount;

        area
    }
}
