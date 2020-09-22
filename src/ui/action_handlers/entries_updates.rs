use super::common::*;

use crate::{entry::EntryIdentifier, ui::util::parent_child_types};

use std::collections::HashMap;

pub async fn action_handler(msg: &Letter, state: &mut UIState) -> RedrawType {
    // we only need to update page entries if entries changed
    match msg {
        Letter::Redraw | Letter::EntryRemoved(_) | Letter::EntryUpdate(_, _) | Letter::ChangePage(_) => {}

        Letter::Hide => {
            if let Some(selected) = state.page_entries.get(state.selected) {
                state.entries.hide(selected);
            }
        }
        _ => {
            return RedrawType::None;
        }
    };

    let last_sel = state.page_entries.get(state.selected);

    let (p, _) = parent_child_types(state.current_page);
    let entries_changed = state.page_entries.set(
        state
            .current_page
            .generate_page(&state.entries, &state.ui_mode)
            .map(|x| *x.0)
            .collect::<Vec<EntryIdentifier>>(),
        p,
    );

    match state.ui_mode {
        UIMode::MoveEntry(ident, _) => {
            if let Some(i) = state
                .page_entries
                .iter_entries()
                .position(|&x| x == ident)
            {
                state.selected = i;
            }
        }
        _ => {
            if let Some(i) = state
                .page_entries
                .iter_entries()
                .position(|&x| Some(x) == last_sel)
            {
                state.selected = i;
            }
        }
    };

    if entries_changed {
        DISPATCH
            .event(Letter::CreateMonitors(
                if state.current_page != PageType::Cards {
                    monitor_list(state)
                } else {
                    HashMap::new()
                },
            ))
            .await;

        RedrawType::Entries
    } else {
        RedrawType::None
    }
}

fn monitor_list(state: &mut UIState) -> HashMap<EntryIdentifier, Option<u32>> {
    let mut monitors = HashMap::new();
    state.page_entries.iter_entries().for_each(|ident| {
        if let Some(entry) = state.entries.get(ident) {
            monitors.insert(
                EntryIdentifier::new(entry.entry_type, entry.index),
                entry.monitor_source(&state.entries),
            );
        }
    });

    log::error!("{:?}", monitors);

    monitors
}
