use super::common::*;

use crate::{
    entry::EntryIdentifier,
    models::context_menus::{self, ContextMenuEffect},
    ui::util::parent_child_types,
};

pub async fn action_handler(msg: &Action, state: &mut RSState) -> RedrawType {
    match msg.clone() {
        Action::EntryRemoved(ident) => {
            if state.page_entries.get(state.selected) == Some(ident) {
                state.ui_mode = UIMode::Normal;
                return RedrawType::Full;
            }
        }
        Action::MoveUp(how_much) => {
            state.selected_context =
                max(state.selected_context as i32 - how_much as i32, 0) as usize;
            return RedrawType::ContextMenu;
        }
        Action::MoveDown(how_much) => {
            state.selected_context = min(
                state.selected_context + how_much as usize,
                state.context_options.len() - 1,
            );
            return RedrawType::ContextMenu;
        }
        Action::CloseContextMenu => {
            state.ui_mode = UIMode::Normal;
            return RedrawType::Full;
        }
        Action::OpenContextMenu => {
            if state.selected >= state.page_entries.len() {
                return RedrawType::None;
            }

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
                ContextMenuEffect::MoveEntry => {
                    let (parent_type, _) = parent_child_types(state.current_page);
                    let entry_ident = state.page_entries.get(state.selected).unwrap();
                    let entry_parent = EntryIdentifier::new(
                        parent_type,
                        state.entries.get(&entry_ident).unwrap().parent.unwrap(),
                    );
                    let parent_ident = match state.entries.find(|(&i, _)| i == entry_parent) {
                        Some((i, _)) => *i,
                        None => EntryIdentifier::new(parent_type, 0),
                    };
                    state.ui_mode = UIMode::MoveEntry(entry_ident, parent_ident);
                    return RedrawType::Full;
                }
            };
        }
        _ => {}
    };

    RedrawType::None
}
