use crate::{actor_system::Ctx, models::{RSState, PAStatus, UIMode}};

pub fn handle(msg: &PAStatus, state: &mut RSState, ctx: &Ctx) {
    match msg {
        PAStatus::PulseAudioDisconnected => {
            state.reset();
        }
        PAStatus::RetryIn(time) => {
            state.change_ui_mode(UIMode::RetryIn(*time));
        }
        PAStatus::ConnectToPulseAudio => {
            state.change_ui_mode(UIMode::Normal);
        }
    }
}
