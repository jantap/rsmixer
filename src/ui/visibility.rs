use super::{parent_child_types, util::Rect, EntryIdentifier, EntryType, Y_PADDING};
use std::cmp::{max, min};

#[derive(PartialEq, Copy, Clone)]
pub enum EntrySpaceLvl {
    Empty,
    Parent,
    ParentNoChildren,
    MidChild,
    LastChild,
}
