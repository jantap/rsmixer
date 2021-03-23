use std::collections::HashSet;

#[derive(PartialEq, Debug, Clone)]
pub struct Redraw {
    pub entries: bool,
    pub peak_volume: Option<usize>,
    pub resize: bool,
    pub affected_entries: HashSet<usize>,
    pub context_menu: bool,
}
impl Default for Redraw {
    fn default() -> Self {
        Self {
            entries: false,
            peak_volume: None,
            resize: false,
            affected_entries: HashSet::new(),
            context_menu: false,
        }
    }
}

impl Redraw {
    pub fn reset(&mut self) {
        *self = Redraw::default();
    }
    pub fn anything(&self) -> bool{
        self.entries || self.peak_volume.is_some() || self.resize || self.context_menu || !self.affected_entries.is_empty()
    }
}
