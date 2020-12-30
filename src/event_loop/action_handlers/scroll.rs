use super::common::*;

use crate::{ui::util::Y_PADDING, RSError};

use std::cmp::{max, min};

pub async fn scroll_handler(msg: &Action, state: &mut RSState) -> Result<RedrawType, RSError> {
    let (_, term_h) = crossterm::terminal::size()?;

    match msg {
        Action::EntryRemoved(_)
        | Action::EntryUpdate(_, _)
        | Action::Redraw
        | Action::ChangePage(_) => {
            state.page_entries.reflow_scroll(term_h - *Y_PADDING, true);
        }
        Action::MoveUp(_) | Action::MoveDown(_) => {}
        _ => {
            return Ok(RedrawType::None);
        }
    };

    if state.page_entries.len() == 0 {
        if state.selected != 0 || state.scroll != 0 {
            state.scroll = 0;
            state.selected = 0;

            return Ok(RedrawType::Entries);
        }

        return Ok(RedrawType::None);
    }

    let new_selected = min(state.page_entries.len() - 1, max(0, state.selected));
    let new_scroll = state.page_entries.visibility[new_selected];

    if new_selected != state.selected || new_scroll != state.scroll {
        state.scroll = new_scroll;
        state.selected = new_selected;
        Ok(RedrawType::Entries)
    } else {
        Ok(RedrawType::None)
    }
}
