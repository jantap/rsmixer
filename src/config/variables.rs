use super::{PulseAudio, RsMixerConfig};

pub struct Variables {
	pub pa_retry_time: u64,
	pub pa_disable_live_volume: bool,
	pub pa_rate: u32,
	pub pa_frag_size: u32,
}

impl Variables {
	pub fn new(config: &RsMixerConfig) -> Self {
		let def = PulseAudio::default();
		let pulse = match &config.pulse_audio {
			Some(p) => p,
			None => &def,
		};

		Self {
			pa_retry_time: pulse.retry_time(),
			pa_rate: pulse.rate(),
			pa_frag_size: pulse.frag_size(),
			pa_disable_live_volume: pulse.disable_live_volume(),
		}
	}
}
