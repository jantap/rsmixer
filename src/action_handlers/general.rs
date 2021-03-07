use super::common::*;

use std::{collections::HashMap, ops::Deref};

pub async fn action_handler(msg: &Action, state: &mut RSState, ctx: &Ctx) {
    match msg.clone() {
        Action::Redraw => {
            state.redraw.resize = true;
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

            entry.inherit_area(&state.entries);

            state.entries.insert(ident, entry);
        }
        Action::PeakVolumeUpdate(ident, peak) => {
            if let Some(play) = state.entries.get_play_entry_mut(&ident) {
                if (play.peak - peak).abs() < f32::EPSILON {
                    return;
                }
                play.peak = peak;

                if let Some(i) = state.page_entries.iter_entries().position(|&i| ident == i) {
                    state.redraw.peak_volume = Some(i);
                }
            }
        }
        Action::ChangePage(page) => {
            state.current_page = page;
            state.ui_mode = UIMode::Normal;
            state.redraw.resize = true;
        }
        Action::PulseAudioDisconnected => {
            ctx.send_to("pulseaudio", Action::CreateMonitors(HashMap::new()));
            *state = RSState::default();
            state.redraw.resize = true;
        }
        Action::RetryIn(time) => {
            state.ui_mode = UIMode::RetryIn(time);
            state.redraw.resize = true;
        }
        Action::ConnectToPulseAudio => {
            state.ui_mode = UIMode::Normal;
            state.redraw.resize = true;
        }
        _ => {}
    };
}
