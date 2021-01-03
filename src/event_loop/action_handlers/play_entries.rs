use super::common::*;

pub async fn action_handler(msg: &Action, state: &mut RSState) -> RedrawType {
    match msg.clone() {
        Action::RequestMute => {
            if state.selected < state.page_entries.len() {
                let mute = match state
                    .entries
                    .get(&state.page_entries.get(state.selected).unwrap())
                {
                    Some(e) => e.play_entry.as_ref().unwrap().mute,
                    None => {
                        return RedrawType::None;
                    }
                };
                DISPATCH
                    .event(Action::MuteEntry(
                        state.page_entries.get(state.selected).unwrap(),
                        !mute,
                    ))
                    .await;
            }
        }
        Action::RequstChangeVolume(how_much) => {
            if state.selected < state.page_entries.len() {
                if let Some(entry) = state
                    .entries
                    .get_mut(&state.page_entries.get(state.selected).unwrap())
                {
                    let mut vols = entry.play_entry.as_ref().unwrap().volume;
                    let avg = vols.avg().0;

                    let base_delta = (volume::Volume::NORMAL.0 as f32 - volume::Volume::MUTED.0 as f32) / 100.0;

                    let current_percent = ((avg - volume::Volume::MUTED.0) as f32 / base_delta).round() as u32;
                    let target_percent = current_percent as i16 + how_much;

                    let target = if target_percent < 0 { volume::Volume::MUTED.0 }
                        else if target_percent == 100 { volume::Volume::NORMAL.0 }
                        else if target_percent >= 150 { (volume::Volume::NORMAL.0 as f32 * 1.5) as u32 }
                        else if target_percent < 100 { volume::Volume::MUTED.0 + target_percent as u32 * base_delta as u32 }
                        else { volume::Volume::NORMAL.0 + (target_percent - 100) as u32 * base_delta as u32 };

                    for v in vols.get_mut() {
                        v.0 = target;
                    }
                    DISPATCH
                        .event(Action::SetVolume(
                            state.page_entries.get(state.selected).unwrap(),
                            vols,
                        ))
                        .await;
                }
            }
        }
        _ => {}
    };
    RedrawType::None
}
