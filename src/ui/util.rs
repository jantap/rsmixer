use crate::{
    entry::{EntrySpaceLvl, EntryType},
    models::PageType,
    STYLES,
    RSError,
};

use crossterm::style::{Attribute, ContentStyle};

use lazy_static::lazy_static;

use std::{io::Write, io};

use crossterm::{cursor::Hide, execute};

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
    pub fn x(&self, x: u16) -> Self {
        Self::new(x, self.y, self.width, self.height)
    }
    pub fn y(&self, y: u16) -> Self {
        Self::new(self.x, y, self.width, self.height)
    }
    pub fn w(&self, w: u16) -> Self {
        Self::new(self.x, self.y, w, self.height)
    }
    pub fn h(&self, h: u16) -> Self {
        Self::new(self.x, self.y, self.width, h)
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
    if lvl == EntrySpaceLvl::Card {
        1
    } else if lvl == EntrySpaceLvl::ParentNoChildren || lvl == EntrySpaceLvl::LastChild {
        4
    } else {
        3
    }
}

pub fn parent_child_types(page: PageType) -> (EntryType, EntryType) {
    match page {
        PageType::Output => (EntryType::Sink, EntryType::SinkInput),
        PageType::Input => (EntryType::Source, EntryType::SourceOutput),
        PageType::Cards => (EntryType::Card, EntryType::Card),
    }
}

pub fn prepare_terminal() -> Result<io::Stdout, RSError> {
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;
    execute!(stdout, Hide)?;

    Ok(stdout)
}

pub fn clean_terminal() -> Result<(), RSError> {
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::cursor::Show,
        crossterm::terminal::LeaveAlternateScreen
    )?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}


pub async fn terminal_too_small<W: Write>(stdout: &mut W) -> Result<(), RSError> {
    let (w, h) = crossterm::terminal::size()?;
    execute!(stdout, crossterm::cursor::MoveTo(0, 0))?;
    let x = get_style("normal").apply(format!(
        "terminal too small{}",
        (0..w * h - 18).map(|_| " ").collect::<String>()
    ));
    write!(stdout, "{}", x)?;
    stdout.flush()?;
    return Ok(());
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
