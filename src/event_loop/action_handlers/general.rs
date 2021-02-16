use super::common::*;

use std::{collections::HashMap, ops::Deref};

pub async fn action_handler(msg: &Action, state: &mut RSState) {
    match msg.clone() {
        Action::Redraw => {
            state.redraw.resize = true;
            state.redraw.full = true;
        }
        Action::EntryRemoved(ident) => {
            state.entries.remove(&ident);
        }
        Action::EntryUpdate(ident, entry) => {
            let mut entry = entry.deref().to_owned();

            if entry.needs_redraw(&state.entries) {
                if let Some(i) = state
                    .page_entries
                    .iter_entries()
                    .position(|id| *id == entry.entry_ident)
                {
                    state.redraw.affected_entries.insert(i);
                }
            }
            if let EntryKind::CardEntry(card) = &mut entry.entry_kind {
                if let Some(old_card) = state.entries.get_card_entry(&ident) {
                    card.area = old_card.area;
                }
            }

            if let EntryKind::PlayEntry(play) = &mut entry.entry_kind {
                if let Some(old_play) = state.entries.get_play_entry(&ident) {
                    play.area = old_play.area;
                    play.volume_bar = old_play.volume_bar;
                    play.peak_volume_bar = old_play.peak_volume_bar;
                }
            }
            state.entries.insert(ident, entry);
        }
        Action::PeakVolumeUpdate(ident, peak) => {
            if ident.entry_type == EntryType::Card {
                return;
            }
            if let Some(play) = state.entries.get_play_entry_mut(&ident) {
                if (play.peak - peak).abs() < f32::EPSILON {
                    return;
                }
                play.peak = peak;
            }
            if let Some(i) = state.page_entries.iter_entries().position(|&i| ident == i) {
                state.redraw.peak_volume = Some(i);
            }
        }
        Action::ChangePage(page) => {
            state.current_page = page;
            state.ui_mode = UIMode::Normal;
            state.redraw.full = true;
        }
        Action::PADisconnected => {
            DISPATCH.event(Action::CreateMonitors(HashMap::new())).await;
            *state = RSState::default();
            state.redraw.full = true;
        }
        Action::RetryIn(time) => {
            state.ui_mode = UIMode::RetryIn(time);
            state.redraw.full = true;
        }
        Action::ConnectToPA => {
            state.ui_mode = UIMode::Normal;
            state.redraw.full = true;
        }
        _ => {}
    };
}
