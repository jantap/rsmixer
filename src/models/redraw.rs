use std::collections::HashSet;

#[derive(PartialEq, Debug, Clone)]
pub struct Redraw {
    pub full: bool,
    pub entries: bool,
    pub partial_entries: Option<HashSet<usize>>,
    pub peak_volume: Option<usize>,
    pub mode: bool,
    pub resize: bool,
    pub affected_entries: HashSet<usize>,
}
impl Default for Redraw {
    fn default() -> Self {
        Self {
            full: false,
            entries: false,
            partial_entries: None,
            peak_volume: None,
            mode: false,
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
