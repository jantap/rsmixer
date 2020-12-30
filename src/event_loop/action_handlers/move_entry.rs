use super::common::*;

pub async fn action_handler(msg: &Action, state: &mut RSState) -> RedrawType {
    match msg.clone() {
        Action::PeakVolumeUpdate(ident, peak) => {
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
            RedrawType::None
        }
        Action::MoveUp(how_much) => {
            if let UIMode::MoveEntry(_, _) = state.ui_mode {
                if state.page_entries.entries.len() < 2 {
                    return RedrawType::None;
                }
                let l = (state.page_entries.len() - 1) as i32;
                let selected = (state.selected - 1) as i32;

                let mut j = selected - how_much as i32;

                if j < 0 {
                    j = j.abs() % l;
                    j = l - j;
                }

                if j >= selected {
                    j += 1;
                }

                let entry_ident = state.page_entries.get(state.selected).unwrap();
                let new_parent = state.page_entries.get(j as usize).unwrap();
                state.ui_mode = UIMode::MoveEntry(entry_ident, new_parent);

                DISPATCH.event(Action::Redraw).await;
            }

            RedrawType::None
        }
        Action::MoveDown(how_much) => {
            if let UIMode::MoveEntry(_, _) = state.ui_mode {
                if state.page_entries.entries.len() < 2 {
                    return RedrawType::None;
                }
                let l = state.page_entries.len() - 1;
                let selected = state.selected - 1;

                let mut j = (selected + how_much as usize) % l;

                if j >= selected {
                    j += 1;
                }

                let entry_ident = state.page_entries.get(state.selected).unwrap();
                let new_parent = state.page_entries.get(j as usize).unwrap();
                state.ui_mode = UIMode::MoveEntry(entry_ident, new_parent);

                DISPATCH.event(Action::Redraw).await;
            }

            RedrawType::None
        }
        Action::OpenContextMenu => match state.ui_mode {
            UIMode::MoveEntry(ident, parent) => {
                state.ui_mode = UIMode::Normal;
                DISPATCH
                    .event(Action::MoveEntryToParent(ident, parent))
                    .await;
                RedrawType::Full
            }
            _ => RedrawType::None,
        },
        _ => RedrawType::None,
    }
}
