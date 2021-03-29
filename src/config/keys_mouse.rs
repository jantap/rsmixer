use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};

use crate::{models::InputEvent, RsError};

pub fn try_string_to_event(key: &str) -> Result<InputEvent, RsError> {
	let s = String::from(key).to_lowercase();
	let mut modifiers = KeyModifiers::empty();

	let parts = s.split('+').collect::<Vec<_>>();

	for &p in parts.iter().take(parts.len() - 1) {
		match p {
			"shift" => modifiers |= KeyModifiers::SHIFT,
			"ctrl" => modifiers |= KeyModifiers::CONTROL,
			"alt" => modifiers |= KeyModifiers::ALT,
			_ => return Err(RsError::KeyCodeError(String::from(key))),
		};
	}

	let code = *parts.last().unwrap();

	if let Ok(kind) = try_string_to_mouseevent(code) {
		Ok(InputEvent::mouse(kind, modifiers))
	} else {
		try_string_to_keyevent(key, code, modifiers)
	}
}

pub fn try_string_to_mouseevent(code: &str) -> Result<MouseEventKind, RsError> {
	match code {
		"scroll_down" => Ok(MouseEventKind::ScrollDown),
		"scroll_up" => Ok(MouseEventKind::ScrollUp),
		"mouse_right" => Ok(MouseEventKind::Up(MouseButton::Right)),
		"mouse_middle" => Ok(MouseEventKind::Up(MouseButton::Middle)),
		_ => Err(RsError::KeyCodeError(code.to_string())),
	}
}

pub fn try_string_to_keyevent(
	key: &str,
	code: &str,
	mut modifiers: KeyModifiers,
) -> Result<InputEvent, RsError> {
	let code = match code {
		"backspace" => KeyCode::Backspace,
		"enter" => KeyCode::Enter,
		"left" => KeyCode::Left,
		"right" => KeyCode::Right,
		"up" => KeyCode::Up,
		"down" => KeyCode::Down,
		"home" => KeyCode::Home,
		"end" => KeyCode::End,
		"pageup" => KeyCode::PageUp,
		"pagedown" => KeyCode::PageDown,
		"tab" => {
			if modifiers.contains(KeyModifiers::SHIFT) {
				modifiers = !modifiers ^ !KeyModifiers::SHIFT;
				KeyCode::BackTab
			} else {
				KeyCode::Tab
			}
		}
		"backtab" => KeyCode::BackTab,
		"delete" => KeyCode::Delete,
		"insert" => KeyCode::Insert,
		"null" => KeyCode::Null,
		"esc" => KeyCode::Esc,
		_ => match code.len() {
			1 => {
				let big_c = code.to_uppercase().chars().next().unwrap();
				let c = code.chars().next().unwrap();
				if modifiers.contains(KeyModifiers::SHIFT)
					&& KeyCode::Char(c) != KeyCode::Char(big_c)
				{
					KeyCode::Char(big_c)
				} else {
					KeyCode::Char(c)
				}
			}
			2 => {
				if let Ok(f) = code[1..code.len()].parse::<u8>() {
					KeyCode::F(f)
				} else {
					return Err(RsError::KeyCodeError(String::from(key)));
				}
			}
			_ => return Err(RsError::KeyCodeError(String::from(key))),
		},
	};

	Ok(InputEvent::key(code, modifiers))
}
