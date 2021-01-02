use super::common::*;

use crate::BINDINGS;

use crossterm::event::KeyEvent;

pub async fn action_handler(key_event: KeyEvent, state: &mut RSState) {
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
        if state.ui_mode == UIMode::ContextMenu {
            actions.retain(|action| *action != Action::ExitSignal);
        } else {
            actions.retain(|action| *action != Action::CloseContextMenu);
        }
    }
}
