use std::convert::TryFrom;

use anyhow::Result;
use crossterm::event::{Event, MouseButton, MouseEvent, MouseEventKind};

use super::volume_input_edit;
use crate::{
	actor_system::Ctx,
	entry::{EntryIdentifier, EntryKind},
	models::{InputEvent, PageType, RSState, UIMode, UserAction, UserInput},
	ui::{Rect, Scrollable},
	BINDINGS,
};

pub fn handle(input: &UserInput, state: &RSState, ctx: &Ctx) -> Result<()> {
	let input_event = InputEvent::try_from(input.event)?;
	let mut actions;

	if let Some(bindings) = (*BINDINGS).get().get_vec(&input_event) {
		actions = bindings.clone();

		handle_conflicting_bindings(&mut actions, state);

		if let Event::Mouse(mouse_event) = input.event {
			handle_mouse_bindings(&mut actions, mouse_event, state);
		}
	} else {
		actions = Vec::new();

		if let Event::Mouse(mouse_event) = input.event {
			handle_unbindable_mouse_actions(&mut actions, mouse_event, state);
		}
	}

	if state.ui_mode == UIMode::InputVolumeValue {
		if let Event::Key(key_event) = input.event {
			volume_input_edit::handle(&mut actions, &key_event, state)?;
		}
	}

	for action in actions {
		ctx.send_to("event_loop", action);
	}

	Ok(())
}

fn handle_unbindable_mouse_actions(
	actions: &mut Vec<UserAction>,
	mouse_event: MouseEvent,
	state: &RSState,
) {
	let mouse_pos = Rect::new(mouse_event.column, mouse_event.row, 1, 1);
	match (&state.ui_mode, mouse_event.kind) {
		(UIMode::Help, MouseEventKind::Up(_)) => {
			if !mouse_pos.intersects(&state.help.window.area) {
				actions.push(UserAction::CloseContextMenu);
			}
		}
		(UIMode::Normal, MouseEventKind::Up(MouseButton::Left)) => {
			let (ident, page_type) = find_collisions(mouse_event, state);

			if let Some(pt) = page_type {
				actions.push(UserAction::ChangePage(pt));
			}
			if let Some(ident) = ident {
				let new_selected = state
					.page_entries
					.iter_entries()
					.position(|i| *i == ident)
					.unwrap_or_else(|| state.page_entries.selected());

				if state.page_entries.selected() == new_selected {
					actions.push(UserAction::OpenContextMenu(None));
				} else {
					actions.push(UserAction::SetSelected(new_selected));
				}
			}
		}
		(UIMode::Normal, MouseEventKind::ScrollUp) => {
			if mouse_pos.y == 0 {
				actions.push(UserAction::CyclePages(-1));
			}
		}
		(UIMode::Normal, MouseEventKind::ScrollDown) => {
			if mouse_pos.y == 0 {
				actions.push(UserAction::CyclePages(1));
			}
		}
		(UIMode::ContextMenu, MouseEventKind::Up(MouseButton::Left)) => {
			if state.context_menu.area.intersects(&mouse_pos) {
				if let Some(i) = state
					.context_menu
					.visible_range(state.context_menu.area.height)
					.enumerate()
					.find(|(index, _)| {
						mouse_event.row == state.context_menu.area.y + (*index) as u16
					})
					.map(|(_, i)| i)
				{
					if i == state.context_menu.selected() {
						actions.push(UserAction::Confirm);
					} else {
						actions.push(UserAction::SetSelected(i));
					}
				}
			} else if !state.context_menu.tool_window.area.intersects(&mouse_pos) {
				actions.push(UserAction::CloseContextMenu);
			}
		}
		(UIMode::ContextMenu, MouseEventKind::ScrollUp) => {
			if state.context_menu.area.intersects(&mouse_pos) {
				actions.push(UserAction::MoveUp(1));
			}
		}
		(UIMode::ContextMenu, MouseEventKind::ScrollDown) => {
			if state.context_menu.area.intersects(&mouse_pos) {
				actions.push(UserAction::MoveDown(1));
			}
		}
		_ => {}
	}
}

fn find_collisions(
	mouse_event: MouseEvent,
	state: &RSState,
) -> (Option<EntryIdentifier>, Option<PageType>) {
	let mouse_event_rect = Rect::new(mouse_event.column, mouse_event.row, 1, 1);

	let mut ident = None;
	let mut page_type = None;

	if mouse_event_rect.y > 0 {
		for entry in state
			.page_entries
			.visible_range(state.ui.entries_area.height)
			.filter_map(|i| state.page_entries.get(i))
			.filter_map(|ident| state.entries.get(&ident))
		{
			let area = match &entry.entry_kind {
				EntryKind::CardEntry(card) => card.area,
				EntryKind::PlayEntry(play) => play.area,
			};

			if area.intersects(&mouse_event_rect) {
				ident = Some(EntryIdentifier::new(entry.entry_type, entry.index));
				break;
			}
		}
	} else {
		let mut cur_x = 1;
		for (i, pn) in state.ui.pages_names.iter().enumerate() {
			if mouse_event_rect.x > cur_x && mouse_event_rect.x < cur_x + pn.len() as u16 {
				page_type = Some(PageType::from(i as i8));
				break;
			}
			cur_x += pn.len() as u16 + 3;
		}
	}
	(ident, page_type)
}

fn handle_mouse_bindings(actions: &mut Vec<UserAction>, mouse_event: MouseEvent, state: &RSState) {
	let (ident, _) = find_collisions(mouse_event, state);

	if ident.is_none() {
		return;
	}

	for a in actions {
		match a {
			UserAction::RequstChangeVolume(value, _) => {
				*a = UserAction::RequstChangeVolume(*value, ident);
			}
			UserAction::RequestMute(_) => {
				*a = UserAction::RequestMute(ident);
			}
			UserAction::OpenContextMenu(_) => {
				*a = UserAction::OpenContextMenu(ident);
			}
			UserAction::Hide(_) => {
				*a = UserAction::Hide(ident);
			}
			_ => {}
		}
	}
}

fn handle_conflicting_bindings(actions: &mut Vec<UserAction>, state: &RSState) {
	if actions.len() == 1 {
		return;
	}

	if actions.contains(&UserAction::RequestQuit) && actions.contains(&UserAction::CloseContextMenu)
	{
		if let UIMode::ContextMenu
		| UIMode::Help
		| UIMode::InputVolumeValue
		| UIMode::MoveEntry(_, _) = state.ui_mode
		{
			actions.retain(|action| *action != UserAction::RequestQuit);
		} else {
			actions.retain(|action| *action != UserAction::CloseContextMenu);
		}
	}

	if actions.contains(&UserAction::Confirm)
		&& actions.contains(&UserAction::OpenContextMenu(None))
	{
		if let UIMode::MoveEntry(_, _) | UIMode::ContextMenu | UIMode::InputVolumeValue =
			state.ui_mode
		{
			actions.retain(|action| *action != UserAction::OpenContextMenu(None));
		} else {
			actions.retain(|action| *action != UserAction::Confirm);
		}
	}

	if actions.contains(&UserAction::MoveLeft) {
		if let UIMode::ContextMenu | UIMode::Help = state.ui_mode {
			actions.retain(|action| *action == UserAction::MoveLeft);
		} else {
			actions.retain(|action| *action != UserAction::MoveLeft);
		}
	}

	if actions.contains(&UserAction::MoveRight) {
		if let UIMode::ContextMenu | UIMode::Help = state.ui_mode {
			actions.retain(|action| *action == UserAction::MoveRight);
		} else {
			actions.retain(|action| *action != UserAction::MoveRight);
		}
	}
}
