use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::models::{RSState, UserAction};

pub fn handle(actions: &mut Vec<UserAction>, input: &KeyEvent, state: &RSState) -> Result<()> {
	let confirm = actions.iter().any(|a| *a == UserAction::Confirm);
	let close_context_menu = actions.iter().any(|a| *a == UserAction::CloseContextMenu);

	if confirm {
		actions.clear();
		actions.push(UserAction::Confirm);
		return Ok(());
	}
	if close_context_menu {
		actions.clear();
		actions.push(UserAction::CloseContextMenu);
		return Ok(());
	}

	let new_input_value = match input.code {
		KeyCode::Char(x @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
			Some(add_char(x, state)?)
		}
		KeyCode::Backspace => Some(remove_char(state)?),
		KeyCode::Left => Some(move_cursor(state, -1)?),
		KeyCode::Right => Some(move_cursor(state, 1)?),
		_ => None,
	};

	if let Some((value, cursor)) = new_input_value {
		actions.clear();
		actions.push(UserAction::ChangeVolumeInputValue(value, cursor));
	}

	Ok(())
}

fn remove_char(state: &RSState) -> Result<(String, u8)> {
	let value = state.input_exact_volume.value.clone();
	let cursor = state.input_exact_volume.cursor as usize;

	if cursor == 0 {
		Ok((value, cursor as u8))
	} else {
		let value = format!("{}{}", &value[0..(cursor - 1)], &value[cursor..]);

		Ok((value, cursor as u8 - 1))
	}
}

fn move_cursor(state: &RSState, val: i8) -> Result<(String, u8)> {
	let value = state.input_exact_volume.value.clone();
	let cursor = state.input_exact_volume.cursor as i8;

	if cursor < -val || cursor > value.len() as i8 - val {
		Ok((value, cursor as u8))
	} else {
		Ok((value, (cursor + val) as u8))
	}
}

fn add_char(c: char, state: &RSState) -> Result<(String, u8)> {
	let value = state.input_exact_volume.value.clone();
	let cursor = state.input_exact_volume.cursor as usize;

	if value.len() == 3 {
		Ok((value, cursor as u8))
	} else {
		let value = format!("{}{}{}", &value[0..cursor], c, &value[cursor..]);

		Ok((value, cursor as u8 + 1))
	}
}
