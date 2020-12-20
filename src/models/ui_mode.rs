use crate::entry::EntryIdentifier;

#[derive(PartialEq, Debug)]
pub enum UIMode {
    Normal,
    ContextMenu,
    Help,
    MoveEntry(EntryIdentifier, EntryIdentifier),
    RetryIn(u64),
}
