use crate::entry::EntryIdentifier;

#[derive(PartialEq)]
pub enum UIMode {
    Normal,
    ContextMenu,
    Help,
    MoveEntry(EntryIdentifier, EntryIdentifier),
    RetryIn(u64),
}
