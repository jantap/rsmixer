use super::{common::*, play_entries};

use std::collections::HashSet;

pub async fn action_handler(msg: &Letter, state: &mut RSState) -> RedrawType {
    let mut redraw = normal_handler(msg, state).await;

    if state.current_page != PageType::Cards {
        play_entries::action_handler(msg, state)
            .await
            .apply(&mut redraw);
    }

    redraw
}

async fn normal_handler(msg: &Letter, state: &mut RSState) -> RedrawType {
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
            let mut affected = HashSet::new();
            affected.insert(state.selected);
            state.selected = max(state.selected as i32 - how_much as i32, 0) as usize;
            affected.insert(state.selected);

            return RedrawType::PartialEntries(affected);
        }
        Letter::MoveDown(how_much) => {
            let mut affected = HashSet::new();
            affected.insert(state.selected);
            state.selected = min(state.selected + how_much as usize, state.page_entries.len());
            affected.insert(state.selected);

            return RedrawType::PartialEntries(affected);
        }
        Letter::CyclePages(which_way) => {
            DISPATCH
                .event(Letter::ChangePage(PageType::from(
                    i8::from(state.current_page) + which_way,
                )))
                .await;
            return RedrawType::None;
        }
        Letter::OpenContextMenu => {
            if state.selected < state.page_entries.len() {
                if let Some(entry) = state
                    .entries
                    .get(&state.page_entries.get(state.selected).unwrap())
                {
                    state.ui_mode = UIMode::ContextMenu;
                    state.context_options = context_menu(entry);

                    if entry.entry_type == EntryType::Card {
                        if let Some(index) = entry.card_entry.as_ref().unwrap().selected_profile {
                            state.selected_context = index;
                        }
                    } else {
                        state.selected_context = 0;
                    }

                    return RedrawType::ContextMenu;
                }
            }
        }
        Letter::ShowHelp => {
            state.ui_mode = UIMode::Help;
            return RedrawType::Help;
        }
        _ => {}
    };
    RedrawType::None
}
