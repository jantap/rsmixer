use super::common::*;

use crate::{
    entry::EntryIdentifier, models::context_menus::ContextMenuEffect, ui::util::parent_child_types,
};

use crate::ui::Scrollable;

pub async fn action_handler(msg: &Action, state: &mut RSState) {
    match msg.clone() {
        Action::EntryRemoved(ident) => {
            if state.context_menu.entry_ident == ident {
                state.change_ui_mode(UIMode::Normal);
            }
        }
        Action::MoveUp(how_much) => {
            state.context_menu.up(how_much as usize);
            state.context_menu.horizontal_scroll = 0;
        }
        Action::MoveDown(how_much) => {
            state.context_menu.down(how_much as usize);
            state.context_menu.horizontal_scroll = 0;
        }
        Action::MoveLeft => {
            if state.context_menu.horizontal_scroll == 0 {
                return;
            }

            state.context_menu.horizontal_scroll -= 1;
        }
        Action::MoveRight => {
            if state.context_menu.horizontal_scroll < state.context_menu.max_horizontal_scroll() {
                state.context_menu.horizontal_scroll += 1;
            }
        }
        Action::CloseContextMenu => {
            state.change_ui_mode(UIMode::Normal);
        }
        Action::Confirm => {
            let selected = match state.page_entries.get_selected() {
                Some(ident) => ident,
                None => {
                    state.change_ui_mode(UIMode::Normal);

                    return;
                }
            };

            let answer = state.context_menu.resolve(selected).await;

            match answer {
                ContextMenuEffect::None => {
                    state.change_ui_mode(UIMode::Normal);
                }
                ContextMenuEffect::MoveEntry => {
                    let (parent_type, _) = parent_child_types(state.current_page);
                    let entry_ident = selected;

                    if let Some(parent_id) = state.entries.get_play_entry(&entry_ident).unwrap().parent {
                        let entry_parent = EntryIdentifier::new(parent_type, parent_id);
                        let parent_ident = match state.entries.find(|(&i, _)| i == entry_parent) {
                            Some((i, _)) => *i,
                            None => EntryIdentifier::new(parent_type, 0),
                        };

                        state.change_ui_mode(UIMode::MoveEntry(entry_ident, parent_ident));
                    }
                }
            };
        }
        _ => {}
    };
}
