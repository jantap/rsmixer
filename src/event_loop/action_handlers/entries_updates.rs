use super::common::*;

use crate::{
    entry::{EntryIdentifier, EntryKind, HiddenStatus},
    ui::util::parent_child_types,
    unwrap_or_return,
};

use screen_buffer_ui::Scrollable;

use std::collections::{HashMap, HashSet};

pub async fn action_handler(msg: &Action, state: &mut RSState) {
    // we only need to update page entries if entries changed
    match msg {
        Action::Redraw
        | Action::EntryRemoved(_)
        | Action::EntryUpdate(_, _)
        | Action::ChangePage(_) => {}

        Action::Hide(Some(ident)) => {
            state.entries.hide(*ident);
        }
        Action::Hide(None) => {
            if let Some(ident) = state.page_entries.get_selected() {
                state.entries.hide(ident);
            }
        }
        _ => {
            return;
        }
    };

    let last_sel = state.page_entries.get(state.page_entries.selected());

    let (p, c) = parent_child_types(state.current_page);

    if p != EntryType::Card && c != EntryType::Card {
        let mut parents = HashSet::new();
        state.entries.iter_type(c).for_each(|(_, e)| {
            if let EntryKind::PlayEntry(play) = &e.entry_kind {
                parents.insert(play.parent);
            }
        });

        for (_, p_e) in state.entries.iter_type_mut(p) {
            if let EntryKind::PlayEntry(play) = &mut p_e.entry_kind {
                play.hidden = match parents.get(&Some(p_e.index)) {
                    Some(_) => HiddenStatus::HiddenKids,
                    None => HiddenStatus::NoKids,
                };
            }
        }
    }

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
            if let Some(i) = state.page_entries.iter_entries().position(|&x| x == ident) {
                state.page_entries.set_selected(i);
            }
        }
        _ => {
            if let Some(i) = state
                .page_entries
                .iter_entries()
                .position(|&x| Some(x) == last_sel)
            {
                state.page_entries.set_selected(i);
            }
        }
    };

    if entries_changed {
        DISPATCH
            .event(Action::CreateMonitors(
                if state.current_page != PageType::Cards {
                    monitor_list(state)
                } else {
                    HashMap::new()
                },
            ))
            .await;

        state.redraw.entries = true;
        state.redraw.resize = true;
    }
}

fn monitor_list(state: &mut RSState) -> HashMap<EntryIdentifier, Option<u32>> {
    let mut monitors = HashMap::new();
    state.page_entries.iter_entries().for_each(|ident| {
        if let Some(entry) = state.entries.get(ident) {
            monitors.insert(
                EntryIdentifier::new(entry.entry_type, entry.index),
                entry.monitor_source(&state.entries),
            );
        }
    });

    monitors
}
