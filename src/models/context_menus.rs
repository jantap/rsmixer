use crate::{
	actor_system::Ctx,
	entry::{Entry, EntryIdentifier, EntryKind, EntryType},
	models::PulseAudioAction,
	scrollable,
	ui::{widgets::ToolWindowWidget, Rect, Scrollable},
};

#[derive(PartialEq, Clone)]
pub enum ContextMenuOption {
	MoveToEntry(EntryIdentifier, String),
	ChangeCardProfile(String, String),
	Kill,
	Move,
	Suspend,
	Resume,
	SetAsDefault,
}

impl From<ContextMenuOption> for String {
	fn from(option: ContextMenuOption) -> Self {
		match option {
			ContextMenuOption::MoveToEntry(_, s) => s,
			ContextMenuOption::ChangeCardProfile(_, s) => s,
			ContextMenuOption::Kill => "Kill".into(),
			ContextMenuOption::Move => "Move".into(),
			ContextMenuOption::Suspend => "Suspend".into(),
			ContextMenuOption::Resume => "Resume".into(),
			ContextMenuOption::SetAsDefault => "Set as default".into(),
		}
	}
}

pub enum ContextMenuEffect {
	None,
	MoveEntry,
}

scrollable!(
	ContextMenu,
	fn selected(&self) -> usize {
		self.selected
	},
	fn len(&self) -> usize {
		self.options.len()
	},
	fn set_selected(&mut self, selected: usize) -> bool {
		if selected < self.options.len() {
			self.selected = selected;
			true
		} else {
			false
		}
	},
	fn element_height(&self, _index: usize) -> u16 {
		1
	}
);

pub struct ContextMenu {
	pub options: Vec<ContextMenuOption>,
	selected: usize,
	pub horizontal_scroll: usize,
	pub area: Rect,
	pub entry_ident: EntryIdentifier,
	pub tool_window: ToolWindowWidget,
}

impl ContextMenu {
	pub fn new(entry: &Entry) -> Self {
		let play = match &entry.entry_kind {
			EntryKind::PlayEntry(play) => Some(play),
			EntryKind::CardEntry(_) => None,
		};
		let card = match &entry.entry_kind {
			EntryKind::PlayEntry(_) => None,
			EntryKind::CardEntry(card) => Some(card),
		};
		let options: Vec<ContextMenuOption> = match entry.entry_type {
			EntryType::Source | EntryType::Sink => vec![
				if play.unwrap().suspended {
					ContextMenuOption::Resume
				} else {
					ContextMenuOption::Suspend
				},
				ContextMenuOption::SetAsDefault,
			],
			EntryType::SinkInput => vec![ContextMenuOption::Move, ContextMenuOption::Kill],
			EntryType::SourceOutput => vec![],
			EntryType::Card => card
				.unwrap()
				.profiles
				.iter()
				.map(|p| {
					ContextMenuOption::ChangeCardProfile(p.name.clone(), p.description.clone())
				})
				.collect(),
		};

		Self {
			options,
			selected: 0,
			horizontal_scroll: 0,
			area: Rect::default(),
			tool_window: ToolWindowWidget::default(),
			entry_ident: EntryIdentifier::new(entry.entry_type, entry.index),
		}
	}

	pub fn resolve(&self, ident: EntryIdentifier, ctx: &Ctx) -> ContextMenuEffect {
		match &self.options[self.selected] {
			ContextMenuOption::Move => {
				return ContextMenuEffect::MoveEntry;
			}
			ContextMenuOption::MoveToEntry(entry, _) => {
				ctx.send_to(
					"pulseaudio",
					PulseAudioAction::MoveEntryToParent(ident, *entry),
				);
			}
			ContextMenuOption::ChangeCardProfile(name, _) => {
				ctx.send_to(
					"pulseaudio",
					PulseAudioAction::ChangeCardProfile(ident, name.clone()),
				);
			}
			ContextMenuOption::Suspend => {
				ctx.send_to("pulseaudio", PulseAudioAction::SetSuspend(ident, true));
			}
			ContextMenuOption::Resume => {
				ctx.send_to("pulseaudio", PulseAudioAction::SetSuspend(ident, false));
			}
			ContextMenuOption::Kill => {
				ctx.send_to("pulseaudio", PulseAudioAction::KillEntry(ident));
			}
			_ => {}
		};

		ContextMenuEffect::None
	}

	pub fn max_horizontal_scroll(&self) -> usize {
		let (start, end) = self.visible_start_end(self.area.height);
		let longest = self
			.options
			.iter()
			.skip(start)
			.take(end - start)
			.map(|o| String::from(o.clone()).len())
			.max();

		match longest {
			None => 0,
			Some(l) => l / self.area.width as usize,
		}
	}
}
impl Default for ContextMenu {
	fn default() -> Self {
		Self {
			options: Vec::new(),
			selected: 0,
			horizontal_scroll: 0,
			area: Rect::default(),
			tool_window: ToolWindowWidget::default(),
			entry_ident: EntryIdentifier::new(EntryType::Sink, 0),
		}
	}
}
