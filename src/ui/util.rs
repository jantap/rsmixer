use crate::entry::{Entries, Entry, EntryIdentifier, EntrySpaceLvl, EntryType};
use crate::STYLES;
use crossterm::style::Attribute;
use crossterm::style::ContentStyle;
use lazy_static::lazy_static;
use std::fmt::Display;

lazy_static! {
    pub static ref Y_PADDING: u16 = 4;
}

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

#[derive(PartialEq, Clone, Hash, Copy, Debug)]
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
    let mut n = name.to_string();
    let mut bold = false;
    if let Some(i) = name.find('.') {
        if name.chars().skip(i + 1).collect::<String>() == "bold" {
            bold = true;
            n = name.chars().take(i).collect::<String>();
        }
    }
    let mut s = match STYLES.get().get(&n[..]) {
        Some(s) => s.clone(),
        None => ContentStyle::default(),
    };

    if bold {
        s = s.attribute(Attribute::Bold);
    }
    s
}

pub fn entry_height(lvl: EntrySpaceLvl) -> u16 {
    if lvl == EntrySpaceLvl::ParentNoChildren || lvl == EntrySpaceLvl::LastChild {
        4
    } else {
        3
    }
}

pub fn parent_child_types(page: PageType) -> (EntryType, EntryType) {
    if page == PageType::Output {
        (EntryType::Sink, EntryType::SinkInput)
    } else {
        (EntryType::Source, EntryType::SourceOutput)
    }
}

#[macro_export]
macro_rules! draw_rect {
    ($stdout:expr, $char:expr, $rect:expr, $style:expr) => {
        let s = (0..$rect.width).map(|_| $char).collect::<String>();
        for y in $rect.y..$rect.y + $rect.height {
            execute!($stdout, crossterm::cursor::MoveTo($rect.x, y))?;
            write!($stdout, "{}", $style.apply(s.clone()))?;
        }
    };
}

#[macro_export]
macro_rules! draw_range {
    ($stdout:expr, $char:expr, $xrange:expr, $yrange:expr, $style:expr) => {
        let s = ($xrange).map(|_| $char).collect::<String>();
        let x = ($xrange).next().unwrap();
        for y in $yrange {
            execute!($stdout, crossterm::cursor::MoveTo(x, y))?;
            write!($stdout, "{}", $style.apply(s.clone()))?;
        }
    };
}

#[macro_export]
macro_rules! draw_at {
    ($stdout:expr, $char:expr, $x:expr, $y:expr, $style:expr) => {
        execute!($stdout, crossterm::cursor::MoveTo($x, $y))?;
        write!($stdout, "{}", $style.apply($char))?;
    };
}

#[macro_export]
macro_rules! repeat_string {
    ($str:expr, $times:expr) => {
        (0..$times).map(|_| $str).collect::<String>()
    };
}
