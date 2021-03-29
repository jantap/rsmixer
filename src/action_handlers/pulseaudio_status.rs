use crate::models::{PAStatus, RSState, UIMode};

pub fn handle(msg: &PAStatus, state: &mut RSState) {
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
