use std::collections::BTreeMap;

use super::{CardEntry, Entry, EntryIdentifier, EntryKind, EntryType, PlayEntry};

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
    pub fn iter_type_mut<'a>(
        &'a mut self,
        entry_type: EntryType,
    ) -> Box<dyn Iterator<Item = (&EntryIdentifier, &mut Entry)> + 'a> {
        Box::new(
            self.0
                .iter_mut()
                .filter(move |(ident, _)| ident.entry_type == entry_type),
        )
    }
    pub fn get(&self, entry_ident: &EntryIdentifier) -> Option<&Entry> {
        self.0.get(entry_ident)
    }
    pub fn get_mut(&mut self, ident: &EntryIdentifier) -> Option<&mut Entry> {
        self.0.get_mut(ident)
    }
    pub fn _get_mut(&mut self, ident: EntryIdentifier) -> Option<&mut Entry> {
        self.0.get_mut(&ident)
    }
    pub fn get_card_entry(&self, entry_ident: &EntryIdentifier) -> Option<&CardEntry> {
        match self.0.get(entry_ident) {
            Some(e) => match &e.entry_kind {
                EntryKind::CardEntry(p) => Some(p),
                _ => None,
            },
            None => None,
        }
    }
    pub fn get_play_entry(&self, entry_ident: &EntryIdentifier) -> Option<&PlayEntry> {
        match self.0.get(entry_ident) {
            Some(e) => match &e.entry_kind {
                EntryKind::PlayEntry(p) => Some(p),
                _ => None,
            },
            None => None,
        }
    }
    pub fn get_play_entry_mut(&mut self, entry_ident: &EntryIdentifier) -> Option<&mut PlayEntry> {
        match self.0.get_mut(entry_ident) {
            Some(e) => match &mut e.entry_kind {
                EntryKind::PlayEntry(p) => Some(p),
                _ => None,
            },
            None => None,
        }
    }
    pub fn position<P>(&mut self, predicate: P) -> Option<usize>
    where
        P: FnMut((&EntryIdentifier, &Entry)) -> bool,
    {
        self.0.iter().position(predicate)
    }
    pub fn find<P>(&mut self, predicate: P) -> Option<(&EntryIdentifier, &Entry)>
    where
        P: FnMut(&(&EntryIdentifier, &Entry)) -> bool,
    {
        self.0.iter().find(predicate)
    }
    pub fn remove(&mut self, ident: &EntryIdentifier) -> Option<Entry> {
        self.0.remove(ident)
    }
    pub fn insert(&mut self, entry_ident: EntryIdentifier, val: Entry) -> Option<Entry> {
        self.0.insert(entry_ident, val)
    }
    pub fn hide(&mut self, ident: EntryIdentifier) {
        let (entry_type, index, parent) = if let Some(entry) = self.0.get(&ident) {
            (entry.entry_type, entry.index, entry.parent())
        } else {
            return;
        };

        match entry_type {
            EntryType::Sink | EntryType::Source => {
                let desired = if entry_type == EntryType::Sink {
                    EntryType::SinkInput
                } else {
                    EntryType::SourceOutput
                };

                self.0
                    .iter_mut()
                    .filter(|(i, e)| {
                        (i.entry_type == desired && e.parent() == Some(index))
                            || (i.entry_type == entry_type && e.index == index)
                    })
                    .for_each(|(_, e)| e.negate_hidden(e.entry_type));
            }
            EntryType::SinkInput | EntryType::SourceOutput => {
                let (desired, parent_type) = if entry_type == EntryType::SinkInput {
                    (EntryType::SinkInput, EntryType::Sink)
                } else {
                    (EntryType::SourceOutput, EntryType::Source)
                };
                self.0
                    .iter_mut()
                    .filter(|(ident, e)| {
                        (ident.entry_type == desired && e.parent() == parent)
                            || (ident.entry_type == parent_type && Some(e.index) == parent)
                    })
                    .for_each(|(_, e)| e.negate_hidden(e.entry_type));
            }
            _ => {}
        }
    }
}
