use super::{EntrySpaceLvl, HiddenStatus};

use crate::ui::{widgets::VolumeWidget, Rect};

use pulse::volume::ChannelVolumes;

#[derive(PartialEq, Clone, Debug)]
pub struct PlayEntry {
    pub peak: f32,
    pub mute: bool,
    pub volume: ChannelVolumes,
    pub monitor_source: Option<u32>,
    pub sink: Option<u32>,
    pub volume_bar: VolumeWidget,
    pub peak_volume_bar: VolumeWidget,
    pub suspended: bool,
    pub area: Rect,
    pub name: String,
    pub is_selected: bool,
    pub position: EntrySpaceLvl,
    pub hidden: HiddenStatus,
    pub parent: Option<u32>,
}
impl Eq for PlayEntry {}
