use super::common::*;

use std::{collections::HashMap, ops::Deref};

pub async fn action_handler(msg: &Action, state: &mut RSState) -> RedrawType {
    match msg.clone() {
        Action::Redraw => {
            return RedrawType::Full;
        }
        Action::EntryRemoved(ident) => {
            state.entries.remove(&ident);
        }
        Action::EntryUpdate(ident, entry) => {
            state.entries.insert(ident, entry.deref().to_owned());
        }
        Action::ChangePage(page) => {
            state.current_page = page;
            state.ui_mode = UIMode::Normal;
            return RedrawType::Full;
        }
        Action::CloseContextMenu => {
            if state.ui_mode == UIMode::Help {
                state.ui_mode = UIMode::Normal;
                return RedrawType::Full;
            }
        }
        Action::PADisconnected => {
            DISPATCH.event(Action::CreateMonitors(HashMap::new())).await;
            *state = RSState::default();
            return RedrawType::Full;
        }
        Action::RetryIn(time) => {
            state.ui_mode = UIMode::RetryIn(time);
            return RedrawType::Full;
        }
        Action::ConnectToPA => {
            state.ui_mode = UIMode::Normal;
            return RedrawType::Full;
        }
        Action::InputVolumeValue => {
            state.ui_mode = UIMode::InputVolumeValue;
            return RedrawType::Entries;
        }
        _ => {}
    };

    RedrawType::None
}
