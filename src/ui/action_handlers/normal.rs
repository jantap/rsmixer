use super::common::*;

pub async fn action_handler(msg: &Letter, state: &mut UIState) -> RedrawType {
    match msg.clone() {
        Letter::EntryUpdate(ident, _) => {
            if state.page_entries.iter_entries().any(|&i| i == ident) {
                return RedrawType::Entries;
            }
        }
        Letter::PeakVolumeUpdate(ident, peak) => {
            if ident.entry_type == EntryType::Card {
                return RedrawType::None;
            }
            if let Some(e) = state.entries.get_mut(&ident) {
                let play = e.play_entry.as_mut().unwrap();
                if (play.peak - peak).abs() < f32::EPSILON {
                    return RedrawType::None;
                }
                play.peak = peak;
            }
            if state.page_entries.iter_entries().any(|&i| i == ident) {
                return RedrawType::PeakVolume(ident);
            }
        }
        Letter::MoveUp(how_much) => {
            state.selected = max(state.selected as i32 - how_much as i32, 0) as usize;
            return RedrawType::Entries;
        }
        Letter::MoveDown(how_much) => {
            state.selected = min(state.selected + how_much as usize, state.page_entries.len());
            return RedrawType::Entries;
        }
        Letter::OpenContextMenu => {
            if state.selected < state.page_entries.len() {
                if let Some(entry) = state
                    .entries
                    .get(&state.page_entries.get(state.selected).unwrap())
                {
                    state.ui_mode = UIMode::ContextMenu;
                    state.context_options = context_menu(entry);
                    return RedrawType::ContextMenu;
                }
            }
        }
        _ => {}
    };
    RedrawType::None
}
