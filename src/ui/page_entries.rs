use super::{entry_height, EntryIdentifier, EntrySpaceLvl, EntryType, Rect, Y_PADDING};

pub struct PageEntries {
    entries: Vec<EntryIdentifier>,
    last_term_h: u16,
    pub lvls: Vec<EntrySpaceLvl>,
    visibility: Vec<usize>,
}

impl PageEntries {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            last_term_h: 0,
            lvls: Vec::new(),
            visibility: Vec::new(),
        }
    }

    pub fn iter_entries(&self) -> std::slice::Iter<EntryIdentifier> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn get(&self, i: usize) -> EntryIdentifier {
        self.entries[i]
    }

    pub fn reflow_scroll(&mut self, force: bool) {
        let (_, h) = crossterm::terminal::size().unwrap();

        if force || h != self.last_term_h {
            self.last_term_h = h;

            self.visibility = Vec::new();

            let h = h - *Y_PADDING;
            let mut scroll_needed = 0;
            let mut curh = 0;
            for l in &self.lvls {
                if entry_height(*l) + curh > h {
                    scroll_needed += 1;
                    curh = entry_height(*l);
                }

                self.visibility.push(scroll_needed);
            }
        }
    }

    pub fn adjust_scroll(&mut self, scroll: &mut usize, selected: &mut usize) -> bool {
        self.reflow_scroll(false);

        let sel = (*selected).clone();
        let scr = (*scroll).clone();

        if *selected >= self.len() {
            if self.len() > 0 {
                *selected = self.len() - 1
            } else {
                *selected = 0;
            };
        }

        *scroll = if *selected < self.visibility.len() {
            self.visibility[*selected]
        } else {
            0
        };

        !(sel == *selected && scr == *scroll)
    }

    pub fn is_entry_visible(&self, index: usize, scroll: usize) -> Option<Rect> {
        let (w, _) = crossterm::terminal::size().unwrap();

        if self.visibility[index] != scroll {
            return None;
        }

        let mut he = 0;
        for i in 0..index {
            if self.visibility[i] == scroll {
                he += entry_height(self.lvls[i]);
            }
        }

        return Some(Rect::new(2, 2 + he, w - 4, entry_height(self.lvls[index])));
    }

    pub fn visible_range_with_lvl<'a>(
        &'a self,
        scroll: usize,
        parent_type: EntryType,
    ) -> Box<dyn Iterator<Item = (usize, EntrySpaceLvl)> + 'a> {
        let start = match self.visibility.iter().position(|&x| x == scroll) {
            Some(s) => s,
            None => 0,
        };
        let end = match self.visibility.iter().rposition(|&x| x == scroll) {
            Some(s) => s + 1,
            None => 0,
        };

        Box::new((start..end).map(move |x| -> (usize, EntrySpaceLvl) { (x, self.lvls[x]) }))
    }

    pub fn set(&mut self, vs: Vec<EntryIdentifier>, parent_type: EntryType) {
        self.lvls = Vec::new();

        for index in 0..vs.len() {
            if vs[index].entry_type == parent_type {
                if index + 1 >= vs.len() || vs[index + 1].entry_type == parent_type {
                    self.lvls.push(EntrySpaceLvl::ParentNoChildren);
                } else {
                    self.lvls.push(EntrySpaceLvl::Parent);
                }
            } else {
                if index + 1 >= vs.len() || vs[index + 1].entry_type == parent_type {
                    self.lvls.push(EntrySpaceLvl::LastChild);
                } else {
                    self.lvls.push(EntrySpaceLvl::MidChild);
                }
            }
        }

        self.reflow_scroll(true);

        self.entries = vs;
    }
}
