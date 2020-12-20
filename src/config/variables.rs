use super::RsMixerConfig;

pub struct Variables {
    pub pa_retry_time: u64,
}

impl Variables {
    pub fn new(config: &RsMixerConfig) -> Self {
        Self {
            pa_retry_time: config.pa_retry_time.unwrap_or(5),
        }
    }
}
