use super::{
    entries::{Entries, Entry},
    parent_child_types, EntryIdentifier,
};
use crate::STYLES;
use crossterm::style::ContentStyle;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
    pub fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }
}

#[derive(PartialEq, Clone, Hash, Copy)]
pub enum PageType {
    Output,
    Input,
}
impl Eq for PageType {}
impl Display for PageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PageType::Output => "Output",
            PageType::Input => "Input",
        };
        write!(f, "{}", s)
    }
}
impl PageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PageType::Output => "Output",
            PageType::Input => "Input",
        }
    }
    pub fn generate_page<'a>(
        &'a self,
        entries: &'a Entries,
    ) -> Box<dyn Iterator<Item = (&EntryIdentifier, &Entry)> + 'a> {
        let (parent, child) = parent_child_types(*self);

        return Box::new(
            entries
                .iter_type(parent)
                .map(move |(ident, entry)| {
                    std::iter::once((ident, entry)).chain(
                        entries
                            .iter_type(child)
                            .filter(move |(_, e)| e.parent == Some(ident.index)),
                    )
                })
                .flatten(),
        );
    }
}

pub fn get_style(name: &'static str) -> ContentStyle {
    match STYLES.get().get(name) {
        Some(s) => s.clone(),
        None => ContentStyle::default(),
    }
}
