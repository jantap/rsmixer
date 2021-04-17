mod buffer;
mod errors;
mod rect;
mod scrollable;
pub mod util;
pub mod widgets;

use std::io::Write;

use buffer::{Buffer, Pixel, Pixels};
pub use errors::UIError;
pub use rect::Rect;
pub use scrollable::Scrollable;
pub use util::{clean_terminal, entry_height, prepare_terminal};
use widgets::{BlockWidget, Widget};

use crate::{
	models::{PageType, RSState, Style, UIMode},
	prelude::*,
};

pub async fn redraw<W: Write>(stdout: &mut W, state: &mut RSState) -> Result<()> {
	make_changes(state).await?;

	state.ui.buffer.draw_changes(stdout)?;

	Ok(())
}
pub async fn make_changes(state: &mut RSState) -> Result<()> {
	if state.redraw.resize {
		state.ui.terminal_too_small = match resize(state) {
			Ok(()) => false,
			Err(err) => match err.source() {
				Some(source) => match source.downcast_ref::<UIError>() {
					Some(UIError::TerminalTooSmall) => true,
					_ => {
						return Err(err);
					}
				},
				None => {
					return Err(err);
				}
			},
		};
	}

	if state.ui.terminal_too_small {
		state.warning_text.text = "Terminal too small".to_string();
		state.warning_text.render(&mut state.ui.buffer)?;

		return Ok(());
	}

	if let UIMode::RetryIn(time) = state.ui_mode {
		state.warning_text.text = format!("PulseAudio disconnected. Retrying in {}...", time);
		state.warning_text.render(&mut state.ui.buffer)?;

		return Ok(());
	}

	if state.redraw.resize {
		state.ui.border.title_pixels= Some(gen_page_names(state));
		state.ui.border.render(&mut state.ui.buffer)?;
	}

	let only_affected =
		!state.redraw.resize && !state.redraw.entries && !state.redraw.affected_entries.is_empty();

	if state.redraw.resize || state.redraw.entries || only_affected {
		let indexes_to_redraw = state
			.page_entries
			.visible_range(state.ui.entries_area.height)
			.filter(|i| !only_affected || state.redraw.affected_entries.get(i).is_some())
			.collect::<Vec<_>>();

		for i in &indexes_to_redraw {
			let ident = state.page_entries.get(*i).unwrap();
			if let Some(entry) = state.entries.get_mut(&ident) {
				entry.position = state.page_entries.lvls[*i];
				entry.is_selected = state.page_entries.selected() == *i;

				entry.render(&mut state.ui.buffer)?;
			}
		}

		if !indexes_to_redraw.is_empty()
			&& indexes_to_redraw.last().unwrap() + 1 == state.page_entries.len()
		{
			let last_entry_ident = state
				.page_entries
				.get(*indexes_to_redraw.last().unwrap())
				.unwrap();

			if let Some(entry) = state.entries.get_mut(&last_entry_ident) {
				let area = entry.area();

				let bottom = Rect::new(
					area.x,
					area.y + area.height,
					area.width,
					state.ui.entries_area.height - area.y - area.height,
				);

				state.ui.buffer.rect(bottom, ' ', Style::Normal);
			}
		}
	}

	if let Some(index) = state.redraw.peak_volume {
		if state
			.page_entries
			.visible_range(state.ui.entries_area.height)
			.any(|i| i == index)
		{
			if let Some(play) = state
				.entries
				.get_play_entry_mut(&state.page_entries.get(index).unwrap())
			{
				play.peak_volume_bar
					.volume(play.peak)
					.render(&mut state.ui.buffer)?;
			}
		}
	}

	match state.ui_mode {
		UIMode::Help => state.help.render(&mut state.ui.buffer)?,
		UIMode::ContextMenu => state.context_menu.render(&mut state.ui.buffer)?,
		UIMode::InputVolumeValue => state.input_exact_volume.render(&mut state.ui.buffer)?,
		_ => {}
	};

	Ok(())
}

pub struct UI {
	pub buffer: Buffer,
	pub border: BlockWidget,
	pub entries_area: Rect,
	pub terminal_too_small: bool,
	pub pages_names: Vec<String>,
}

impl Default for UI {
	fn default() -> Self {
		Self {
			buffer: Buffer::default(),
			border: BlockWidget::default().clean_inside(true),
			entries_area: Rect::default(),
			terminal_too_small: false,
			pages_names: vec![
				PageType::Output.to_string(),
				PageType::Input.to_string(),
				PageType::Cards.to_string(),
			],
		}
	}
}

fn resize(state: &mut RSState) -> Result<()> {
	let (x, y) = crossterm::terminal::size()?;
	state.ui.buffer.resize(x, y);

	state.ui.border.resize(Rect::new(
		0,
		0,
		state.ui.buffer.width,
		state.ui.buffer.height,
	))?;

	state.ui.entries_area = Rect::new(2, 2, state.ui.buffer.width - 4, state.ui.buffer.height - 4);
	let mut entry_area = state.ui.entries_area;

	for i in state
		.page_entries
		.visible_range(state.ui.entries_area.height)
	{
		let ent = match state.entries.get_mut(&state.page_entries.get(i).unwrap()) {
			Some(x) => x,
			None => {
				continue;
			}
		};
		ent.position = state.page_entries.lvls[i];

		entry_area = entry_area.h(entry_height(ent.position));

		ent.resize(entry_area)?;

		entry_area.y += entry_height(ent.position);
	}

	state.context_menu.resize(state.ui.entries_area)?;

	if state.ui_mode == UIMode::InputVolumeValue {
		if let Some(ident) = &state.page_entries.get_selected() {
			if let Some(play) = state.entries.get_play_entry(ident) {
				state.input_exact_volume.resize(play.area)?;
			}
		}
	}

	state.help.resize(state.ui.entries_area)?;

	Ok(())
}

fn gen_page_names(state: &mut RSState) -> Pixels{
	if state.ui.buffer.width as usize
		> 2 + state.ui.pages_names.iter().map(|p| p.len()).sum::<usize>() + 6  {
            let style = |i: usize| {
                if i as i8 == state.current_page.into() {
                    Style::Bold
                } else {
                    Style::Normal
                }
            };

            (0..3).fold(Pixels::default(), |pixels, x| pixels.string(style(x), &state.ui.pages_names[x]))
        } else {
            Pixels::default().string(Style::Bold, &state.ui.pages_names[state.page_entries.selected()])
        }
}
