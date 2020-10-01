use crate::entry::EntryIdentifier;

use std::collections::HashSet;

#[derive(PartialEq, Debug, Clone)]
pub enum RedrawType {
    Help,
    Full,
    Entries,
    PartialEntries(HashSet<usize>),
    PeakVolume(EntryIdentifier),
    ContextMenu,
    None,
}
impl Eq for RedrawType {}

impl From<RedrawType> for u32 {
    fn from(redraw: RedrawType) -> u32 {
        match redraw {
            RedrawType::Help => 2000,
            RedrawType::Full => 1000,
            RedrawType::Entries => 500,
            RedrawType::ContextMenu => 500,
            RedrawType::PartialEntries(_) => 400,
            RedrawType::PeakVolume(_) => 100,
            RedrawType::None => 1,
        }
    }
}
impl std::cmp::PartialOrd for RedrawType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for RedrawType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let a = u32::from(self.clone());
        let b = u32::from(other.clone());

        a.cmp(&b)
    }
}

impl RedrawType {
    pub fn take_bigger(&mut self, mut other: Self) {
        if let RedrawType::PartialEntries(p1) = self {
            if let RedrawType::PartialEntries(p2) = other {
                let mut ps = HashSet::new();
                for p in p1.iter() {
                    ps.insert(*p);
                }
                for p in p2.iter() {
                    ps.insert(*p);
                }
                *self = RedrawType::PartialEntries(ps);
                return;
            }
        }
        if self.cmp(&&mut other) == std::cmp::Ordering::Less {
            *self = other;
        }
    }

    pub fn apply(&self, other: &mut Self) {
        if let RedrawType::PartialEntries(p1) = self {
            if let RedrawType::PartialEntries(p2) = other {
                let mut ps = HashSet::new();
                for p in p1.iter() {
                    ps.insert(*p);
                }
                for p in p2.iter() {
                    ps.insert(*p);
                }
                *other = RedrawType::PartialEntries(ps);
                return;
            }
        }
        if self.cmp(other) == std::cmp::Ordering::Greater {
            *other = self.clone();
        }
    }
}

