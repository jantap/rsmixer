use super::EntryType;

use std::cmp::Ordering;

#[derive(Clone, Copy, PartialEq, Hash, Debug)]
pub struct EntryIdentifier {
    pub entry_type: EntryType,
    pub index: u32,
}

impl Eq for EntryIdentifier {}

impl std::cmp::PartialOrd for EntryIdentifier {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for EntryIdentifier {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self == other {
            return std::cmp::Ordering::Equal;
        }

        let result = self.entry_type.cmp(&other.entry_type);
        match result {
            Ordering::Equal => Ordering::Greater,
            _ => result,
        }
    }
}

impl EntryIdentifier {
    pub fn new(entry_type: EntryType, index: u32) -> Self {
        Self { entry_type, index }
    }

    pub fn is_card(&self) -> bool {
        self.entry_type == EntryType::Card
    }
}
