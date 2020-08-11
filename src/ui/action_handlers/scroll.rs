use super::common::*;
use crate::RSError;
use crate::ui::util::Y_PADDING;
use std::cmp::{min, max};

pub async fn scroll_handler(msg: &Letter, state: &mut UIState) -> Result<RedrawType, RSError> {
    let (_, term_h) = crossterm::terminal::size()?;

    match msg {
        Letter::EntryRemoved(_)
        | Letter::EntryUpdate(_, _)
        | Letter::Redraw
        | Letter::ChangePage(_) => {
            state.page_entries.reflow_scroll(term_h - *Y_PADDING, true);
        },
        Letter::MoveUp(_)
        | Letter::MoveDown(_) => {},
        _ => { return Ok(RedrawType::None); },
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
