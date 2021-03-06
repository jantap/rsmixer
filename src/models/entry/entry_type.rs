#[derive(Clone, Copy, PartialEq, Hash, Eq, Debug)]
pub enum EntryType {
    Sink,
    SinkInput,
    Source,
    SourceOutput,
    Card,
}

impl From<EntryType> for u8 {
    fn from(x: EntryType) -> Self {
        match x {
            EntryType::Sink => 1,
            EntryType::Source => 2,
            EntryType::SinkInput => 3,
            EntryType::SourceOutput => 4,
            EntryType::Card => 5,
        }
    }
}

impl std::cmp::PartialOrd for EntryType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for EntryType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let a: u8 = (*self).into();
        let b: u8 = (*other).into();

        a.cmp(&b)
    }
}
