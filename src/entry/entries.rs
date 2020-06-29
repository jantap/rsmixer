use super::Entry;
use super::{EntryIdentifier, EntryType};
use std::collections::BTreeMap;

pub struct Entries(BTreeMap<EntryIdentifier, Entry>);

impl Entries {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
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
}
