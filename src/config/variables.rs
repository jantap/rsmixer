
use super::RsMixerConfig;

use linked_hash_map::LinkedHashMap;

pub enum VarType {
    Usize(usize),
    Bool(bool),
}

pub struct Variables {
    pub pa_retry_time: u64,
}

impl Variables {
    pub fn new(config: &RsMixerConfig) -> Self {
        Self {
            pa_retry_time: match config.pa_retry_time {
                Some(x) => x,
                None => 5,
            }
        }
    }
}
