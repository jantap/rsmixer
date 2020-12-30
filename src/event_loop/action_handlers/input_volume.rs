use super::common::*;

pub async fn action_handler(msg: &Letter, state: &mut RSState) -> RedrawType {
    match msg.clone() {
        Letter::OpenContextMenu => {
            state.ui_mode = UIMode::Normal;

            return RedrawType::Entries;
        }
        Letter::CloseContextMenu => {
            state.ui_mode = UIMode::Normal;

            return RedrawType::Entries;
        }
        _ => {},
    };
    RedrawType::None
}
