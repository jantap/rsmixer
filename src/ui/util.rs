use std::io;

use crossterm::{cursor::Hide, execute};

use crate::{entry::EntrySpaceLvl, ui::UIError};

pub fn entry_height(lvl: EntrySpaceLvl) -> u16 {
	if lvl == EntrySpaceLvl::Card {
		1
	} else if lvl == EntrySpaceLvl::ParentNoChildren || lvl == EntrySpaceLvl::LastChild {
		4
	} else {
		3
	}
}

pub fn prepare_terminal() -> Result<io::Stdout, UIError> {
	let mut stdout = io::stdout();
	crossterm::execute!(
		stdout,
		crossterm::terminal::EnterAlternateScreen,
		crossterm::event::EnableMouseCapture
	)?;
	crossterm::terminal::enable_raw_mode()?;
	execute!(stdout, Hide)?;

	Ok(stdout)
}

pub fn clean_terminal() -> Result<(), UIError> {
	let mut stdout = std::io::stdout();
	crossterm::execute!(
		stdout,
		crossterm::cursor::Show,
		crossterm::terminal::LeaveAlternateScreen,
		crossterm::event::DisableMouseCapture
	)?;
	crossterm::terminal::disable_raw_mode()?;

	Ok(())
}

#[macro_export]
macro_rules! repeat {
	($char:expr, $times:expr) => {
		(0..$times).map(|_| $char).collect::<String>()
	};
}

#[macro_export]
macro_rules! format_text {
    ($char:expr, $($style:expr),*) => {
        {
            let mut v = Vec::new();
            let styles = vec![$($style),*];
            let mut active_style = 0;
            let mut word = $char.chars().peekable();
            while let Some(cur) = word.next() {
                if cur == '{' && word.peek() == Some(&'}') {
                    active_style += 1;
                    word.next();
                    continue;
                }

                v.push(Pixel {
                    style: styles[active_style],
                    text: Some(cur),
                });
            }

            v
        }
    }
}
#[macro_export]
macro_rules! format_text2 {
    ($($x:tt)*) => {
        let res = format_text_intern!(format_args!($($x:tt)*));
        res
    }
}
