use super::common::*;

use crate::unwrap_or_return;

pub async fn action_handler(msg: &Action, state: &mut RSState) {
    match msg.clone() {
        Action::RequestMute(ident) => {
            let ident = ident.unwrap_or(unwrap_or_return!(state.page_entries.get_selected()));

            let mute = match state.entries.get_play_entry(&ident) {
                Some(p) => p.mute,
                None => {
                    return;
                }
            };
            DISPATCH.event(Action::MuteEntry(ident, !mute)).await;
        }
        Action::RequstChangeVolume(how_much, ident) => {
            let ident = ident.unwrap_or(unwrap_or_return!(state.page_entries.get_selected()));

            if let Some(play) = state.entries.get_play_entry_mut(&ident) {
                let mut vols = play.volume;
                let avg = vols.avg().0;

                let base_delta =
                    (volume::Volume::NORMAL.0 as f32 - volume::Volume::MUTED.0 as f32) / 100.0;

                let current_percent =
                    ((avg - volume::Volume::MUTED.0) as f32 / base_delta).round() as u32;
                let target_percent = current_percent as i16 + how_much;

                let target = if target_percent < 0 {
                    volume::Volume::MUTED.0
                } else if target_percent == 100 {
                    volume::Volume::NORMAL.0
                } else if target_percent >= 150 {
                    (volume::Volume::NORMAL.0 as f32 * 1.5) as u32
                } else if target_percent < 100 {
                    volume::Volume::MUTED.0 + target_percent as u32 * base_delta as u32
                } else {
                    volume::Volume::NORMAL.0 + (target_percent - 100) as u32 * base_delta as u32
                };

                for v in vols.get_mut() {
                    v.0 = target;
                }
                DISPATCH.event(Action::SetVolume(ident, vols)).await;
            }
        }
        _ => {}
    };
}
