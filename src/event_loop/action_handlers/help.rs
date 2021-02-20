use super::common::*;

use crate::ui::Scrollable;

pub async fn action_handler(msg: &Action, state: &mut RSState) {
    match msg.clone() {
        Action::MoveUp(how_much) => {
            state.help.up(how_much as usize);
        }
        Action::MoveDown(how_much) => {
            state.help.down(how_much as usize);
        }
        Action::CloseContextMenu => {
            state.ui_mode = UIMode::Normal;
            state.redraw.full = true;
            state.redraw.resize = true;
        }
        _ => {}
    };
}
