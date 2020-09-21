
use crate::RSError;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn try_string_to_keyevent(key: &str) -> Result<KeyEvent, RSError> {
    let s = String::from(key).to_lowercase();
    let mut modifiers = KeyModifiers::empty();

    let parts = s.split("+").collect::<Vec<_>>();


    for &p in parts.iter().take(parts.len() - 1) {
        match p {
            "shift" => modifiers = modifiers | KeyModifiers::SHIFT,
            "ctrl" => modifiers = modifiers | KeyModifiers::CONTROL,
            "alt" => modifiers = modifiers | KeyModifiers::ALT,
            _ => return Err(RSError::KeyCodeError(String::from(key))),
        };
    }

    let code = *parts.last().unwrap();
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
            },
            "backtab" => KeyCode::BackTab,
            "delete" => KeyCode::Delete,
            "insert" => KeyCode::Insert,
            "null" => KeyCode::Null,
            "esc" => KeyCode::Esc,
            _ => {
                match code.len() {
                    1 => {
                        let big_c = code.to_uppercase().chars().next().unwrap();
                        let c = code.chars().next().unwrap();
                        if modifiers.contains(KeyModifiers::SHIFT)
                            && KeyCode::Char(c) != KeyCode::Char(big_c) {
                                KeyCode::Char(big_c)
                        } else {
                            KeyCode::Char(c)
                        }
                    }
                    2 => {
                        if let Ok(f) = code[1..code.len()].parse::<u8>() {
                            KeyCode::F(f)
                        } else {
                            return Err(RSError::KeyCodeError(String::from(key)));
                        }
                    },
                    _ => return Err(RSError::KeyCodeError(String::from(key))),
                }
            }
    };


    Ok(KeyEvent {
        code,
        modifiers,
    })
}

pub fn keyevent_to_string(key_ev: &KeyEvent) -> String {
    let mut key_ev = key_ev.clone();

    if key_ev.code == KeyCode::BackTab {
        key_ev.code = KeyCode::Tab;
        key_ev.modifiers = key_ev.modifiers | KeyModifiers::SHIFT;
    }

    let mut s = "".to_string();
    if key_ev.modifiers.contains(KeyModifiers::CONTROL) {
        s = format!("Ctrl+");
    }
    if key_ev.modifiers.contains(KeyModifiers::SHIFT) {
        s = format!("{}Shift+", s);
    }
    if key_ev.modifiers.contains(KeyModifiers::ALT) {
        s = format!("{}Alt+", s);
    }

    let code = match key_ev.code {
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
    };

    format!("{}{}", s, code)
}
