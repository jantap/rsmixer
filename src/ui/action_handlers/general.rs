use super::common::*;

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
            state.entries.insert(ident, entry.clone());
        }
        Letter::ChangePage(page) => {
            state.current_page = page;
            state.ui_mode = UIMode::Normal;
            return RedrawType::Full;
        }
        _ => {}
    };

    return RedrawType::None;
}
