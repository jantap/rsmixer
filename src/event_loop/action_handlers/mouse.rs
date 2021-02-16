use super::common::*;

use crate::BINDINGS;

use crossterm::event::MouseEvent;

pub async fn action_handler(mouse_event: MouseEvent, state: &mut RSState) {
    if let Some(bindings) = (*BINDINGS).get().get_vec(&key_event) {
        let mut actions = bindings.clone();

        handle_conflicting_bindings(&mut actions, state);

        for action in actions {
            DISPATCH.event(action).await;
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

    if actions.contains(&Action::Confirm) && actions.contains(&Action::OpenContextMenu) {
        if state.ui_mode == UIMode::ContextMenu {
            actions.retain(|action| *action != Action::OpenContextMenu);
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
