use super::common::*;

use crate::{entry::EntryKind, models::ContextMenu};

use screen_buffer_ui::Scrollable;

pub async fn action_handler(msg: &Action, state: &mut RSState) {
    normal_handler(msg, state).await;
}

async fn normal_handler(msg: &Action, state: &mut RSState) {
    match msg.clone() {
        Action::EntryUpdate(ident, _) => {
            if state.page_entries.iter_entries().any(|&i| i == ident) {
                state.redraw.entries = true;
            }
        }
        Action::MoveUp(how_much) => {
            state
                .redraw
                .affected_entries
                .insert(state.page_entries.selected());
            state.page_entries.up(how_much as usize);
            state
                .redraw
                .affected_entries
                .insert(state.page_entries.selected());
        }
        Action::MoveDown(how_much) => {
            state
                .redraw
                .affected_entries
                .insert(state.page_entries.selected());
            state.page_entries.down(how_much as usize);
            state
                .redraw
                .affected_entries
                .insert(state.page_entries.selected());
        }
        Action::CyclePages(which_way) => {
            DISPATCH
                .event(Action::ChangePage(PageType::from(
                    i8::from(state.current_page) + which_way,
                )))
                .await;
        }
        Action::OpenContextMenu => {
            if state.page_entries.selected() < state.page_entries.len() {
                if let Some(entry) = state.entries.get(
                    &state
                        .page_entries
                        .get(state.page_entries.selected())
                        .unwrap(),
                ) {
                    state.ui_mode = UIMode::ContextMenu;
                    state.context_menu = ContextMenu::new(entry);

                    if let EntryKind::CardEntry(card) = &entry.entry_kind {
                        state
                            .context_menu
                            .set_selected(card.selected_profile.unwrap_or(0));
                    }

                    state.redraw.resize = true;
                }
            }
        }
        Action::ShowHelp => {
            state.ui_mode = UIMode::Help;
            state.redraw.resize = true;
        }
        _ => {}
    };
}
