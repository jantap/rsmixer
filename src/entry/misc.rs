#[derive(PartialEq, Copy, Clone, Debug)]
pub enum EntrySpaceLvl {
    Empty,
    Parent,
    ParentNoChildren,
    MidChild,
    LastChild,
    Card,
}

#[derive(Clone, Copy, PartialEq, Hash, Eq, Debug)]
pub enum EntryType {
    Sink,
    SinkInput,
    Source,
    SourceOutput,
    Card,
}

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
        let num = |x| match x {
            EntryType::Sink => 1,
            EntryType::Source => 2,
            EntryType::SinkInput => 3,
            EntryType::SourceOutput => 4,
            EntryType::Card => 5,
        };

        if self.entry_type == other.entry_type && self.index == other.index {
            return std::cmp::Ordering::Equal;
        }

        let a = num(self.entry_type);
        let b = num(other.entry_type);

        if let std::cmp::Ordering::Equal = a.cmp(&b) {
            if self.index > other.index {
                return std::cmp::Ordering::Greater;
            }
            std::cmp::Ordering::Less
        } else {
            a.cmp(&b)
        }
    }
}

impl EntryIdentifier {
    pub fn new(entry_type: EntryType, index: u32) -> Self {
        Self { entry_type, index }
    }
}
