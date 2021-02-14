use super::common::*;

use crate::{
    entry::EntryIdentifier, models::context_menus::ContextMenuEffect, ui::util::parent_child_types,
};

use screen_buffer_ui::Scrollable;

pub async fn action_handler(msg: &Action, state: &mut RSState) {
    match msg.clone() {
        Action::EntryRemoved(ident) => {
            if state.context_menu.entry_ident == ident {
                state.ui_mode = UIMode::Normal;
                state.redraw.full = true;
                state.redraw.resize = true;
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
            state.ui_mode = UIMode::Normal;
            state.redraw.full = true;
            state.redraw.resize = true;
        }
        Action::Confirm => {
            if state.page_entries.selected() >= state.page_entries.len() {
                state.ui_mode = UIMode::ContextMenu;
                state.redraw.full = true;
                state.redraw.resize = true;

                return;
            }

            let answer = state
                .context_menu
                .resolve(state.page_entries.get_selected().unwrap())
                .await;

            match answer {
                ContextMenuEffect::None => {
                    state.ui_mode = UIMode::Normal;
                    state.redraw.full = true;
                    state.redraw.resize = true;
                }
                ContextMenuEffect::MoveEntry => {
                    let (parent_type, _) = parent_child_types(state.current_page);
                    let entry_ident = state.page_entries.get_selected().unwrap();

                    if let Some(parent_id) =
                        state.entries.get_play_entry(&entry_ident).unwrap().parent
                    {
                        let entry_parent = EntryIdentifier::new(parent_type, parent_id);
                        let parent_ident = match state.entries.find(|(&i, _)| i == entry_parent) {
                            Some((i, _)) => *i,
                            None => EntryIdentifier::new(parent_type, 0),
                        };
                        state.ui_mode = UIMode::MoveEntry(entry_ident, parent_ident);
                        state.redraw.full = true;
                        state.redraw.resize = true;
                    }
                }
            };
        }
        _ => {}
    };
}
