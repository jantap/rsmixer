use super::common::*;

use crate::{
    entry::{EntryIdentifier, EntryKind},
    models::InputEvent,
    RsError, BINDINGS,
};

use crate::ui::{Rect, Scrollable};

use std::convert::TryFrom;

use crossterm::event::{Event, MouseButton, MouseEvent, MouseEventKind};

pub async fn action_handler(event: Event, state: &mut RSState) -> Result<(), RsError> {
    let input_event = InputEvent::try_from(event)?;
    let mut actions;

    if let Some(bindings) = (*BINDINGS).get().get_vec(&input_event) {
        actions = bindings.clone();

        handle_conflicting_bindings(&mut actions, state);

        if let Event::Mouse(mouse_event) = event {
            handle_mouse_bindings(&mut actions, mouse_event, state);
        }
    } else {
        actions = Vec::new();

        if let Event::Mouse(mouse_event) = event {
            handle_unbindable_mouse_actions(&mut actions, mouse_event, state);
        }
    }

    for action in actions {
        DISPATCH.event(action).await;
    }

    Ok(())
}

fn handle_unbindable_mouse_actions(
    actions: &mut Vec<Action>,
    mouse_event: MouseEvent,
    state: &mut RSState,
) {
    let mouse_pos = Rect::new(mouse_event.column, mouse_event.row, 1, 1);
    match (&state.ui_mode, mouse_event.kind) {
        (UIMode::Help, MouseEventKind::Up(_)) => {
            if !mouse_pos.intersects(&state.help.window.area) {
                actions.push(Action::CloseContextMenu);
            }
        }
        (UIMode::Normal, MouseEventKind::Up(MouseButton::Left)) => {
            let (ident, page_type) = find_collisions(mouse_event, state);

            if let Some(pt) = page_type {
                actions.push(Action::ChangePage(pt));
            }
            if let Some(ident) = ident {
                let new_selected = state
                    .page_entries
                    .iter_entries()
                    .position(|i| *i == ident)
                    .unwrap_or_else(|| state.page_entries.selected());

                if state.page_entries.selected() == new_selected {
                    actions.push(Action::OpenContextMenu(None));
                } else {
                    state
                        .redraw
                        .affected_entries
                        .insert(state.page_entries.selected());
                    state.redraw.affected_entries.insert(new_selected);
                    state.page_entries.set_selected(new_selected);
                }
            }
        }
        (UIMode::Normal, MouseEventKind::ScrollUp) => {
            if mouse_pos.y == 0 {
                actions.push(Action::CyclePages(-1));
            }
        }
        (UIMode::Normal, MouseEventKind::ScrollDown) => {
            if mouse_pos.y == 0 {
                actions.push(Action::CyclePages(1));
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
                        actions.push(Action::Confirm);
                    } else {
                        state.context_menu.set_selected(i);
                    }
                }
            } else if !state.context_menu.tool_window.area.intersects(&mouse_pos) {
                actions.push(Action::CloseContextMenu);
            }
        }
        (UIMode::ContextMenu, MouseEventKind::ScrollUp) => {
            if state.context_menu.area.intersects(&mouse_pos) {
                state.context_menu.up(1);
            }
        }
        (UIMode::ContextMenu, MouseEventKind::ScrollDown) => {
            if state.context_menu.area.intersects(&mouse_pos) {
                state.context_menu.down(1);
            }
        }
        _ => {}
    }
}

fn find_collisions(
    mouse_event: MouseEvent,
    state: &mut RSState,
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

fn handle_mouse_bindings(actions: &mut Vec<Action>, mouse_event: MouseEvent, state: &mut RSState) {
    let (ident, _) = find_collisions(mouse_event, state);

    if ident.is_none() {
        return;
    }

    for a in actions {
        match a {
            Action::RequstChangeVolume(value, _) => {
                *a = Action::RequstChangeVolume(*value, ident);
            }
            Action::RequestMute(_) => {
                *a = Action::RequestMute(ident);
            }
            Action::OpenContextMenu(_) => {
                *a = Action::OpenContextMenu(ident);
            }
            Action::Hide(_) => {
                *a = Action::Hide(ident);
            }
            _ => {}
        }
    }
}

fn handle_conflicting_bindings(actions: &mut Vec<Action>, state: &mut RSState) {
    if actions.len() == 1 {
        return;
    }

    if actions.contains(&Action::ExitSignal) && actions.contains(&Action::CloseContextMenu) {
        match state.ui_mode {
            UIMode::ContextMenu | UIMode::Help => {
                actions.retain(|action| *action != Action::ExitSignal);
            }
            _ => {
                actions.retain(|action| *action != Action::CloseContextMenu);
            }
        }
    }

    if actions.contains(&Action::Confirm) && actions.contains(&Action::OpenContextMenu(None)) {
        if state.ui_mode == UIMode::ContextMenu {
            actions.retain(|action| *action != Action::OpenContextMenu(None));
        } else {
            actions.retain(|action| *action != Action::Confirm);
        }
    }

    if actions.contains(&Action::MoveLeft) {
        match state.ui_mode {
            UIMode::ContextMenu | UIMode::Help => {
                actions.retain(|action| *action == Action::MoveLeft);
            }
            _ => {
                actions.retain(|action| *action != Action::MoveLeft);
            }
        }
    }

    if actions.contains(&Action::MoveRight) {
        match state.ui_mode {
            UIMode::ContextMenu | UIMode::Help => {
                actions.retain(|action| *action == Action::MoveRight);
            }
            _ => {
                actions.retain(|action| *action != Action::MoveRight);
            }
        }
    }
}
