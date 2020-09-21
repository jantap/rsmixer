use super::{Entry, EntryIdentifier, EntryType};

use std::collections::BTreeMap;

pub struct Entries(BTreeMap<EntryIdentifier, Entry>);

impl Default for Entries {
    fn default() -> Self {
        Self(BTreeMap::new())
    }
}

impl Entries {
    pub fn iter_type<'a>(
        &'a self,
        entry_type: EntryType,
    ) -> Box<dyn Iterator<Item = (&EntryIdentifier, &Entry)> + 'a> {
        Box::new(
            self.0
                .iter()
                .filter(move |(ident, _)| ident.entry_type == entry_type),
        )
    }
    pub fn get(&self, entry_ident: &EntryIdentifier) -> Option<&Entry> {
        self.0.get(entry_ident)
    }
    pub fn get_mut(&mut self, ident: &EntryIdentifier) -> Option<&mut Entry> {
        self.0.get_mut(ident)
    }
    pub fn remove(&mut self, ident: &EntryIdentifier) -> Option<Entry> {
        self.0.remove(ident)
    }
    pub fn insert(&mut self, entry_ident: EntryIdentifier, val: Entry) -> Option<Entry> {
        self.0.insert(entry_ident, val)
    }
    pub fn hide(&mut self, ident: EntryIdentifier) {
        let (entry_type, index, parent) = if let Some(entry) = self.0.get(&ident) {
            (entry.entry_type, entry.index, entry.parent)
        } else {
            return
        };

        match entry_type {
            EntryType::Sink | EntryType::Source => {
                let desired = if entry_type == EntryType::Sink { EntryType::SinkInput } else { EntryType::SourceOutput };
                    self.0.iter_mut().filter(|(i, _)| i.entry_type == desired).filter(|(_, e)| e.parent == Some(index)).for_each(|(_, e)| e.hidden = !e.hidden);
            }
            EntryType::SinkInput | EntryType::SourceOutput => {
                let desired = if entry_type == EntryType::SinkInput { EntryType::SinkInput } else { EntryType::SourceOutput };
                self.0.iter_mut().filter(|(ident, _)| ident.entry_type == desired).filter(|(_, e)| e.parent == parent).for_each(|(_, e)| e.hidden = !e.hidden);
            }
            _ => {}
        }
    }
}
