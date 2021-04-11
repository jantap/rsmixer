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
		crossterm::terminal::LeaveAlternateScreen
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
