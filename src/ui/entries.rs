use crate::{EntryIdentifier, EntryType};
use std::collections::BTreeMap;

#[derive(PartialEq, Clone)]
pub struct Entry {
    pub entry_type: EntryType,
    pub index: u32,
    pub name: String,
    pub peak: f32,
    pub mute: bool,
    pub volume: pulse::volume::ChannelVolumes,
    pub parent: Option<u32>,
}
impl Eq for Entry {}

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
