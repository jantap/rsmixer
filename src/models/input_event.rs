use crate::RSError;

use std::{
    convert::TryFrom,
    fmt::{self, Display},
};

use crossterm::event::{Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind};

#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub enum InputEventKind {
    Mouse(MouseEventKind),
    Key(KeyCode),
}
impl Eq for InputEventKind {}

#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub struct InputEvent {
    pub kind: InputEventKind,
    pub modifiers: KeyModifiers,
}
impl Eq for InputEvent {}

impl InputEvent {
    pub fn key(key_code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self {
            kind: InputEventKind::Key(key_code),
            modifiers,
        }
    }
    pub fn mouse(mouse_kind: MouseEventKind, modifiers: KeyModifiers) -> Self {
        Self {
            kind: InputEventKind::Mouse(mouse_kind),
            modifiers,
        }
    }
}

impl TryFrom<Event> for InputEvent {
    type Error = RSError;
    fn try_from(value: Event) -> Result<Self, Self::Error> {
        match value {
            Event::Key(key) => Ok(InputEvent::key(key.code, key.modifiers)),
            Event::Mouse(mouse) => Ok(InputEvent::mouse(mouse.kind, mouse.modifiers)),
            _ => Err(RSError::KeyCodeError(
                "Event::Redraw cannot be converted to InputEvent".to_string(),
            )),
        }
    }
}

impl Display for InputEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut modifiers = self.modifiers.clone();
        let mut kind = self.kind.clone();

        if let InputEventKind::Key(key) = kind {
            if key == KeyCode::BackTab {
                kind = InputEventKind::Key(KeyCode::Tab);
                modifiers |= KeyModifiers::SHIFT;
            }
        }

        if modifiers.contains(KeyModifiers::CONTROL) {
            write!(f, "Ctrl+")?;
        }
        if modifiers.contains(KeyModifiers::SHIFT) {
            write!(f, "Shift+")?;
        }
        if modifiers.contains(KeyModifiers::ALT) {
            write!(f, "Alt+")?;
        }

        let last = match kind {
            InputEventKind::Key(code) => match code {
                KeyCode::Backspace => "Backspace".to_string(),
                KeyCode::Enter => "Enter".to_string(),
                KeyCode::Left => "Left".to_string(),
                KeyCode::Right => "Right".to_string(),
                KeyCode::Up => "Up".to_string(),
                KeyCode::Down => "Down".to_string(),
                KeyCode::Home => "Home".to_string(),
                KeyCode::End => "End".to_string(),
                KeyCode::PageUp => "PageUp".to_string(),
                KeyCode::PageDown => "PageDown".to_string(),
                KeyCode::Tab => "Tab".to_string(),
                KeyCode::BackTab => "Backtab".to_string(),
                KeyCode::Delete => "Delete".to_string(),
                KeyCode::Insert => "Insert".to_string(),
                KeyCode::Null => "Null".to_string(),
                KeyCode::Esc => "Esc".to_string(),
                KeyCode::F(i) => format!("F{}", i),
                KeyCode::Char(c) => format!("{}", c),
            },
            InputEventKind::Mouse(mouse) => match mouse {
                MouseEventKind::Up(MouseButton::Left) => "MLeft".to_string(),
                MouseEventKind::Up(MouseButton::Right) => "MRight".to_string(),
                MouseEventKind::Up(MouseButton::Middle) => "MMiddle".to_string(),
                MouseEventKind::ScrollUp => "ScrollUp".to_string(),
                MouseEventKind::ScrollDown => "ScrollDown".to_string(),
                _ => "".to_string(),
            },
        };

        write!(f, "{}", last)
    }
}
