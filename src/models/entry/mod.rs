mod card_entry;
mod entries;
mod entry_type;
mod identifier;
mod play_entry;

pub use card_entry::{CardEntry, CardProfile};
pub use entries::Entries;
pub use entry_type::EntryType;
pub use identifier::EntryIdentifier;
pub use play_entry::PlayEntry;
use pulse::volume::ChannelVolumes;

use crate::{
	ui::{widgets::VolumeWidget, Rect},
	unwrap_or_return,
};

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum EntrySpaceLvl {
	Empty,
	Parent,
	ParentNoChildren,
	MidChild,
	LastChild,
	Card,
}

#[derive(PartialEq, Clone, Debug)]
pub enum EntryKind {
	CardEntry(CardEntry),
	PlayEntry(PlayEntry),
}

impl EntryKind {
	pub fn play_entry(&self) -> Option<&PlayEntry> {
		match self {
			Self::PlayEntry(play) => Some(play),
			Self::CardEntry(_) => None,
		}
	}
	pub fn card_entry(&self) -> Option<&CardEntry> {
		match self {
			Self::CardEntry(card) => Some(card),
			Self::PlayEntry(_) => None,
		}
	}
	pub fn play_entry_mut(&mut self) -> Option<&mut PlayEntry> {
		match self {
			Self::PlayEntry(play) => Some(play),
			Self::CardEntry(_) => None,
		}
	}
	pub fn card_entry_mut(&mut self) -> Option<&mut CardEntry> {
		match self {
			Self::CardEntry(card) => Some(card),
			Self::PlayEntry(_) => None,
		}
	}
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
	pub entry_ident: EntryIdentifier,
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
			entry_ident: EntryIdentifier::new(entry_type, index),
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
			entry_ident: EntryIdentifier::new(EntryType::Card, index),
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

	pub fn needs_redraw(&self, entries: &Entries) -> bool {
		match &self.entry_kind {
			EntryKind::CardEntry(card) => {
				let old_card = unwrap_or_return!(entries.get_card_entry(&self.entry_ident), true);
				old_card.name != card.name || old_card.selected_profile != card.selected_profile
			}
			EntryKind::PlayEntry(play) => {
				let old_play = unwrap_or_return!(entries.get_play_entry(&self.entry_ident), true);
				old_play.name != play.name
					|| old_play.mute != play.mute
					|| old_play.volume != play.volume
					|| (play.peak - old_play.peak).abs() < f32::EPSILON
			}
		}
	}

	pub fn inherit_area(&mut self, entries: &Entries) {
		match &mut self.entry_kind {
			EntryKind::CardEntry(card) => {
				if let Some(old_card) = entries.get_card_entry(&self.entry_ident) {
					card.area = old_card.area;
				}
			}
			EntryKind::PlayEntry(play) => {
				if let Some(old_play) = entries.get_play_entry(&self.entry_ident) {
					play.area = old_play.area;
					play.volume_bar = old_play.volume_bar;
					play.peak_volume_bar = old_play.peak_volume_bar;
				}
			}
		};
	}

	pub fn area(&self) -> Rect {
		match &self.entry_kind {
			EntryKind::CardEntry(card) => card.area,
			EntryKind::PlayEntry(play) => play.area,
		}
	}
}
