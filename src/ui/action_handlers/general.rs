use super::common::*;

use std::ops::Deref;

pub async fn action_handler(msg: &Letter, state: &mut UIState) -> RedrawType {
    match msg.clone() {
        Letter::ExitSignal => return RedrawType::Exit,
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
        _ => {}
    };

    RedrawType::None
}
