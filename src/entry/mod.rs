mod entries;
mod misc;

pub use entries::Entries;
pub use misc::{EntryIdentifier, EntrySpaceLvl, EntryType};

use crate::ui::widgets::VolumeWidget;

use screen_buffer_ui::Rect;

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

#[derive(PartialEq, Clone, Debug)]
pub struct CardProfile {
    pub name: String,
    pub description: String,
    #[cfg(any(feature = "pa_v13"))]
    pub available: bool,
    pub area: Rect,
    pub is_selected: bool,
}
impl Eq for CardProfile {}

#[derive(PartialEq, Clone, Debug)]
pub struct CardEntry {
    pub profiles: Vec<CardProfile>,
    pub selected_profile: Option<usize>,
    pub area: Rect,
    pub is_selected: bool,
    pub name: String,
}
impl Eq for CardEntry {}

#[derive(PartialEq, Clone, Debug)]
pub enum EntryKind {
    CardEntry(CardEntry),
    PlayEntry(PlayEntry),
}

#[derive(PartialEq, Clone, Debug, Copy)]
pub enum HiddenStatus {
    Show,
    HiddenKids,
    Hidden,
    NoKids,
}
impl Eq for HiddenStatus {}

impl HiddenStatus {
    pub fn negate(&mut self, entry_type: EntryType) {
        match entry_type {
            EntryType::Source | EntryType::Sink => match self {
                Self::Show => *self = Self::HiddenKids,
                Self::HiddenKids => *self = Self::Show,
                _ => {}
            },
            EntryType::SourceOutput | EntryType::SinkInput => match self {
                Self::Show => *self = Self::Hidden,
                Self::Hidden => *self = Self::Show,
                _ => {}
            },
            _ => {}
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Entry {
    pub entry_type: EntryType,
    pub index: u32,
    pub name: String,
    pub is_selected: bool,
    pub position: EntrySpaceLvl,
    pub entry_kind: EntryKind,
}
impl Eq for Entry {}

impl Entry {
    pub fn negate_hidden(&mut self, entry_type: EntryType) {
        if let EntryKind::PlayEntry(play) = &mut self.entry_kind {
            play.hidden.negate(entry_type);
        }
    }

    pub fn parent(&self) -> Option<u32> {
        if let EntryKind::PlayEntry(play) = &self.entry_kind {
            play.parent
        } else {
            None
        }
    }

    pub fn new_play_entry(
        entry_type: EntryType,
        index: u32,
        name: String,
        parent: Option<u32>,
        mute: bool,
        volume: ChannelVolumes,
        monitor_source: Option<u32>,
        sink: Option<u32>,
        suspended: bool,
    ) -> Self {
        Self {
            entry_type,
            index,
            name: name.clone(),
            is_selected: false,
            position: EntrySpaceLvl::Empty,
            entry_kind: EntryKind::PlayEntry(PlayEntry {
                peak: 0.0,
                mute,
                parent,
                volume,
                monitor_source,
                sink,
                volume_bar: VolumeWidget::default(),
                peak_volume_bar: VolumeWidget::default(),
                suspended,
                area: Rect::default(),
                name,
                is_selected: false,
                position: EntrySpaceLvl::Empty,
                hidden: HiddenStatus::Show,
            }),
        }
    }

    pub fn new_card_entry(
        index: u32,
        name: String,
        profiles: Vec<CardProfile>,
        selected_profile: Option<usize>,
    ) -> Self {
        Self {
            entry_type: EntryType::Card,
            index,
            name: name.clone(),
            is_selected: false,
            position: EntrySpaceLvl::Card,
            entry_kind: EntryKind::CardEntry(CardEntry {
                area: Rect::default(),
                is_selected: false,
                profiles,
                selected_profile,
                name,
            }),
        }
    }

    pub fn calc_area(position: EntrySpaceLvl, mut area: Rect) -> Rect {
        let amount = match position {
            EntrySpaceLvl::Card => 1,
            EntrySpaceLvl::Parent => 2,
            EntrySpaceLvl::ParentNoChildren => 2,
            _ => 5,
        };

        area.x += amount;
        if amount < area.width {
            area.width -= amount;
        } else {
            area.width = 0;
        }

        area
    }

    pub fn monitor_source(&self, entries: &Entries) -> Option<u32> {
        if let EntryKind::PlayEntry(play) = &self.entry_kind {
            match self.entry_type {
                EntryType::SinkInput => {
                    if let Some(sink) = play.sink {
                        match entries.get(&EntryIdentifier::new(EntryType::Sink, sink)) {
                            Some(_) => play.monitor_source,
                            None => None,
                        }
                    } else {
                        None
                    }
                }
                _ => play.monitor_source,
            }
        } else {
            None
        }
    }
}
