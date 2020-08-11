use crate::ui::util::{entry_height, Rect, Y_PADDING};
use crate::{
    entry::{EntryIdentifier, EntrySpaceLvl, EntryType},
    RSError,
};

pub struct PageEntries {
    entries: Vec<EntryIdentifier>,
    pub last_term_h: u16,
    pub lvls: Vec<EntrySpaceLvl>,
    pub visibility: Vec<usize>,
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

    pub fn get(&self, i: usize) -> Option<EntryIdentifier> {
        if i < self.len() {
            Some(self.entries[i])
        } else {
            None
        }
    }

    pub fn reflow_scroll(&mut self, h: u16, force: bool) {
        log::error!("reflow 1");
        if !force && h == self.last_term_h {
            return;
        }
        log::error!("reflow 2");

        self.last_term_h = h;

        self.visibility = Vec::new();

        let mut current_scroll_page = 0;
        let mut current_height = 0;

        self.visibility = self.lvls.iter().map(|&e| {
            current_height += entry_height(e);

            if current_height > h {
                current_scroll_page += 1;
                current_height = 0;
            }

            current_scroll_page
        }).collect();
    }

    pub fn is_entry_visible(&self, index: usize, scroll: usize) -> Result<Option<Rect>, RSError> {
        let (w, _) = crossterm::terminal::size()?;

        if self.visibility[index] != scroll {
            return Ok(None);
        }

        let mut he = 0;
        for i in 0..index {
            if self.visibility[i] == scroll {
                he += entry_height(self.lvls[i]);
            }
        }

        return Ok(Some(Rect::new(
            2,
            2 + he,
            w - 4,
            entry_height(self.lvls[index]),
        )));
    }

    pub fn visible_range_with_lvl<'a>(
        &'a self,
        scroll: usize,
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

    pub fn set(&mut self, vs: Vec<EntryIdentifier>, parent_type: EntryType) -> bool {
        let ret = if vs.len() == self.len() {

            // check if any page entry changed identifier or level
            vs.iter().enumerate().find(|&(i, &e)| e != self.get(i).unwrap() || calc_lvl(parent_type, &vs, i) != self.lvls[i]) != None

        } else {
            true
        };

        if ret {
            self.lvls = Vec::new();

            for index in 0..vs.len() {
                self.lvls.push(calc_lvl(parent_type, &vs, index));
            }

            self.reflow_scroll(self.last_term_h, true);

            self.entries = vs;
        }

        ret
    }
}

fn calc_lvl(parent_type: EntryType, vs: &Vec<EntryIdentifier>, index: usize) -> EntrySpaceLvl {
    if parent_type == EntryType::Card {
        EntrySpaceLvl::Card
    } else if vs[index].entry_type == parent_type {
        if index + 1 >= vs.len() || vs[index + 1].entry_type == parent_type {
            EntrySpaceLvl::ParentNoChildren
        } else {
            EntrySpaceLvl::Parent
        }
    } else {
        if index + 1 >= vs.len() || vs[index + 1].entry_type == parent_type {
            EntrySpaceLvl::LastChild
        } else {
            EntrySpaceLvl::MidChild
        }
    }
}
