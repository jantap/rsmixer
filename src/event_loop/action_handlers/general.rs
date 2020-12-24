use super::common::*;

use std::{collections::HashMap, ops::Deref};

pub async fn action_handler(msg: &Letter, state: &mut RSState) -> RedrawType {
    match msg.clone() {
        Letter::Redraw => {
            return RedrawType::Full;
        }
        Letter::EntryRemoved(ident) => {
            state.entries.remove(&ident);
        }
        Letter::EntryUpdate(ident, entry) => {
            state.entries.insert(ident, entry.deref().to_owned());
        }
        Letter::ChangePage(page) => {
            state.current_page = page;
            state.ui_mode = UIMode::Normal;
            return RedrawType::Full;
        }
        Letter::CloseContextMenu => {
            if state.ui_mode == UIMode::Help {
                state.ui_mode = UIMode::Normal;
                return RedrawType::Full;
            }
        }
        Letter::PADisconnected => {
            DISPATCH.event(Letter::CreateMonitors(HashMap::new())).await;
            *state = RSState::default();
            return RedrawType::Full;
        }
        Letter::RetryIn(time) => {
            state.ui_mode = UIMode::RetryIn(time);
            return RedrawType::Full;
        }
        Letter::ConnectToPA => {
            state.ui_mode = UIMode::Normal;
            return RedrawType::Full;
        }
        _ => {}
    };

    RedrawType::None
}
