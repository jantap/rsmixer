use std::collections::{HashMap, HashSet};

use super::RSState;
use crate::{
    entry::{EntryIdentifier, EntryKind, EntryType, HiddenStatus},
    models::{PageType, PulseAudioAction, UIMode},
    ui::Scrollable,
};

pub fn update(state: &mut RSState) {
    let last_sel = state.page_entries.get_selected();

    let (p, c) = state.current_page.parent_child_types();

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
        let monitors = monitor_list(state);
        state
            .ctx()
            .send_to("pulseaudio", PulseAudioAction::CreateMonitors(monitors));

        state.redraw.resize = true;
    }
}

fn monitor_list(state: &mut RSState) -> HashMap<EntryIdentifier, Option<u32>> {
    let mut monitors = HashMap::new();

    if state.current_page == PageType::Cards {
        return monitors;
    }

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
