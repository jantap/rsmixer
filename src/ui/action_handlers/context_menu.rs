use super::common::*;

use crate::ui::models::context_menus::{self, ContextMenuEffect, ContextMenuOption};

pub async fn action_handler(msg: &Letter, state: &mut UIState) -> RedrawType {
    match msg.clone() {
        Letter::EntryRemoved(ident) => {
            if state.page_entries.get(state.selected) == Some(ident) {
                state.ui_mode = UIMode::Normal;
                return RedrawType::Full;
            }
        }
        Letter::MoveUp(how_much) => {
            state.selected_context =
                max(state.selected_context as i32 - how_much as i32, 0) as usize;
            return RedrawType::ContextMenu;
        }
        Letter::MoveDown(how_much) => {
            state.selected_context = min(
                state.selected_context + how_much as usize,
                state.context_options.len(),
            );
            return RedrawType::ContextMenu;
        }
        Letter::OpenContextMenu => {
            if state.selected < state.page_entries.len() {
                let ans = context_menus::resolve(
                    state.page_entries.get(state.selected).unwrap(),
                    state.context_options[state.selected_context].clone(),
                )
                .await;

                match ans {
                    ContextMenuEffect::None => {
                        state.ui_mode = UIMode::Normal;
                        return RedrawType::Full;
                    }
                    ContextMenuEffect::PresentParents => {
                        let p_type = if state.page_entries.get(state.selected).unwrap().entry_type
                            == EntryType::SinkInput
                        {
                            EntryType::Sink
                        } else {
                            EntryType::Source
                        };

                        state.context_options = state
                            .entries
                            .iter_type(p_type)
                            .map(|(ident, entry)| {
                                ContextMenuOption::MoveToEntry(*ident, entry.name.clone())
                            })
                            .collect();
                        return RedrawType::ContextMenu;
                    }
                };
            }
        }
        _ => {}
    };

    RedrawType::None
}
