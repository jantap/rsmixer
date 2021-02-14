use crate::{
    entry::{EntryIdentifier, EntrySpaceLvl, EntryType},
    ui::util::entry_height,
};

use screen_buffer_ui::{scrollable, Scrollable};

pub struct PageEntries {
    pub entries: Vec<EntryIdentifier>,
    pub last_term_h: u16,
    pub lvls: Vec<EntrySpaceLvl>,
    pub visibility: Vec<usize>,
    selected: usize,
}

impl PageEntries {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            last_term_h: 0,
            lvls: Vec::new(),
            visibility: Vec::new(),
            selected: 0,
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

    pub fn get_selected(&self) -> Option<EntryIdentifier> {
        self.get(self.selected())
    }

    pub fn set(&mut self, vs: Vec<EntryIdentifier>, parent_type: EntryType) -> bool {
        let ret = if vs.len() == self.len() {
            // check if any page entry changed identifier or level
            vs.iter().enumerate().find(|&(i, &e)| {
                e != self.get(i).unwrap() || calc_lvl(parent_type, &vs, i) != self.lvls[i]
            }) != None
        } else {
            true
        };

        if ret {
            self.lvls = Vec::new();

            for index in 0..vs.len() {
                self.lvls.push(calc_lvl(parent_type, &vs, index));
            }

            self.entries = vs;
        }

        ret
    }
}

scrollable!(
    PageEntries,
    fn selected(&self) -> usize {
        self.selected
    },
    fn len(&self) -> usize {
        self.entries.len()
    },
    fn set_selected(&mut self, selected: usize) -> bool {
        if selected < self.entries.len() {
            self.selected = selected;
            true
        } else {
            false
        }
    },
    fn element_height(&self, index: usize) -> u16 {
        if let Some(lvl) = self.lvls.get(index) {
            entry_height(*lvl)
        } else {
            0
        }
    }
);

fn calc_lvl(parent_type: EntryType, vs: &[EntryIdentifier], index: usize) -> EntrySpaceLvl {
    if parent_type == EntryType::Card {
        EntrySpaceLvl::Card
    } else if vs[index].entry_type == parent_type {
        if index + 1 >= vs.len() || vs[index + 1].entry_type == parent_type {
            EntrySpaceLvl::ParentNoChildren
        } else {
            EntrySpaceLvl::Parent
        }
    } else if index + 1 >= vs.len() || vs[index + 1].entry_type == parent_type {
        EntrySpaceLvl::LastChild
    } else {
        EntrySpaceLvl::MidChild
    }
}
