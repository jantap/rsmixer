use super::common::*;

pub async fn action_handler(msg: &Letter, state: &mut UIState) -> RedrawType {
    match msg.clone() {
        Letter::EntryUpdate(ident, _) => {
            if state.page_entries.iter_entries().any(|&i| i == ident) {
                return RedrawType::Entries;
            }
        }
        Letter::PeakVolumeUpdate(ident, peak) => {
            if let Some(e) = state.entries.get_mut(&ident) {
                if e.peak == peak {
                    return RedrawType::None;
                }
                e.peak = peak;
            }
            if state.page_entries.iter_entries().any(|&i| i == ident) {
                return RedrawType::PeakVolume(ident);
            }
        }
        Letter::MoveUp(how_much) => {
            state.selected = max(state.selected as i32 - how_much as i32, 0) as usize;
            return RedrawType::Entries;
        }
        Letter::MoveDown(how_much) => {
            state.selected = min(state.selected + how_much as usize, state.page_entries.len());
            return RedrawType::Entries;
        }
        Letter::RequestMute => {
            if state.selected < state.page_entries.len() {
                let mute = match state
                    .entries
                    .get(&state.page_entries.get(state.selected).unwrap())
                {
                    Some(e) => e.mute,
                    None => {
                        return RedrawType::None;
                    }
                };
                DISPATCH
                    .event(Letter::MuteEntry(
                        state.page_entries.get(state.selected).unwrap(),
                        !mute,
                    ))
                    .await;
            }
        }
        Letter::RequstChangeVolume(how_much) => {
            if state.selected < state.page_entries.len() {
                if let Some(entry) = state
                    .entries
                    .get_mut(&state.page_entries.get(state.selected).unwrap())
                {
                    let mut vols = entry.volume.clone();
                    for v in vols.get_mut() {
                        // @TODO add config
                        // @TODO don't overflow
                        let amount =
                            (volume::VOLUME_NORM.0 as f32 * how_much as f32 / 100.0) as i64;
                        if (v.0 as i64) < volume::VOLUME_MUTED.0 as i64 - amount {
                            v.0 = volume::VOLUME_MUTED.0;
                        } else if (v.0 as i64)
                            > (volume::VOLUME_NORM.0 as f32 * 1.5) as i64 - amount
                        {
                            v.0 = (volume::VOLUME_NORM.0 as f32 * 1.5) as u32;
                        } else {
                            v.0 = (v.0 as i64 + amount) as u32;
                        }
                    }
                    DISPATCH
                        .event(Letter::SetVolume(
                            state.page_entries.get(state.selected).unwrap(),
                            vols,
                        ))
                        .await;
                }
            }
        }
        Letter::OpenContextMenu => {
            if state.selected < state.page_entries.len() {
                if let Some(entry) = state
                    .entries
                    .get(&state.page_entries.get(state.selected).unwrap())
                {
                    state.ui_mode = UIMode::ContextMenu;
                    state.context_options = context_menu(entry);
                    return RedrawType::ContextMenu;
                }
            }
        }
        _ => {}
    };
    return RedrawType::None;
}
