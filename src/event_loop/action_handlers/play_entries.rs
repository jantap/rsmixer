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
                    for v in vols.get_mut() {
                        // @TODO add config
                        // @TODO don't overflow
                        let amount =
                            (volume::Volume::NORMAL.0 as f32 * how_much as f32 / 100.0) as i64;
                        if (v.0 as i64) < volume::Volume::MUTED.0 as i64 - amount {
                            v.0 = volume::Volume::MUTED.0;
                        } else if (v.0 as i64)
                            > (volume::Volume::NORMAL.0 as f32 * 1.5) as i64 - amount
                        {
                            v.0 = (volume::Volume::NORMAL.0 as f32 * 1.5) as u32;
                        } else {
                            v.0 = (v.0 as i64 + amount) as u32;
                        }
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
