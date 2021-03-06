use std::collections::HashSet;

#[derive(PartialEq, Debug, Clone)]
pub struct Redraw {
    pub entries: bool,
    pub peak_volume: Option<usize>,
    pub resize: bool,
    pub affected_entries: HashSet<usize>,
}
impl Default for Redraw {
    fn default() -> Self {
        Self {
            entries: false,
            peak_volume: None,
            resize: false,
            affected_entries: HashSet::new(),
        }
    }
}

impl Redraw {
    pub fn reset(&mut self) {
        *self = Redraw::default();
    }
}
