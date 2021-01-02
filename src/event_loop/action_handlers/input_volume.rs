use super::common::*;

pub async fn action_handler(msg: &Action, state: &mut RSState) -> RedrawType {
    match msg.clone() {
        Action::OpenContextMenu => {
            state.ui_mode = UIMode::Normal;

            return RedrawType::Entries;
        }
        Action::CloseContextMenu => {
            state.ui_mode = UIMode::Normal;

            return RedrawType::Entries;
        }
        _ => {}
    };
    RedrawType::None
}
