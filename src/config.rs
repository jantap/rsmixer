use crate::{RSError, ui::PageType, Letter, Styles};

use std::{convert::TryFrom, collections::HashMap};

use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    style::{Color, ContentStyle},
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RsMixerConfig {
    bindings: HashMap<String, String>,
    colors: HashMap<String, HashMap<String, String>>,
}

impl std::default::Default for RsMixerConfig {
    fn default() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert("q".to_string(), "exit".to_string());
        bindings.insert("j".to_string(), "down(1)".to_string());
        bindings.insert("k".to_string(), "up(1)".to_string());
        bindings.insert("m".to_string(), "mute".to_string());
        bindings.insert("h".to_string(), "lower_volume(5)".to_string());
        bindings.insert("l".to_string(), "raise_volume(5)".to_string());
        bindings.insert("H".to_string(), "lower_volume(15)".to_string());
        bindings.insert("L".to_string(), "raise_volume(15)".to_string());
        bindings.insert("1".to_string(), "show_output".to_string());
        bindings.insert("2".to_string(), "show_input".to_string());
        bindings.insert("3".to_string(), "show_cards".to_string());
        bindings.insert("enter".to_string(), "context_menu".to_string());
        bindings.insert("tab".to_string(), "cycle_pages_forward".to_string());
        bindings.insert("s+tab".to_string(), "cycle_pages_backward".to_string());
        bindings.insert("esc".to_string(), "close_context_menu".to_string());
        let mut styles = HashMap::new();
        let mut c = HashMap::new();
        c.insert("fg".to_string(), "white".to_string());
        c.insert("bg".to_string(), "black".to_string());
        styles.insert("normal".to_string(), c.clone());
        c.insert("fg".to_string(), "black".to_string());
        c.insert("bg".to_string(), "white".to_string());
        styles.insert("inverted".to_string(), c.clone());
        c.insert("fg".to_string(), "grey".to_string());
        c.insert("bg".to_string(), "black".to_string());
        styles.insert("muted".to_string(), c.clone());
        c.insert("fg".to_string(), "red".to_string());
        c.insert("bg".to_string(), "black".to_string());
        styles.insert("red".to_string(), c.clone());
        c.insert("fg".to_string(), "yellow".to_string());
        c.insert("bg".to_string(), "black".to_string());
        styles.insert("orange".to_string(), c.clone());
        c.insert("fg".to_string(), "green".to_string());
        c.insert("bg".to_string(), "black".to_string());
        styles.insert("green".to_string(), c);
        Self {
            bindings,
            colors: styles,
        }
    }
}

impl RsMixerConfig {
    pub fn load(&self) -> Result<(Styles, HashMap<KeyEvent, Letter>), RSError> {
        let mut bindings: HashMap<KeyEvent, Letter> = HashMap::new();

        for (k, c) in &self.bindings {
            bindings.insert(find_keycode(&k)?, Letter::try_from(c.clone())?);
        }

        let mut styles: Styles = HashMap::new();

        for (k, v) in &self.colors {
            let mut c = ContentStyle::new();
            if let Some(q) = v.get("fg") {
                if let Some(color) = find_color(q) {
                    c = c.foreground(color);
                } else {
                    return Err(RSError::InvalidColor(q.clone()));
                }
            }
            if let Some(q) = v.get("bg") {
                if let Some(color) = find_color(q) {
                    c = c.background(color);
                } else {
                    return Err(RSError::InvalidColor(q.clone()));
                }
            }
            styles.insert(k.clone(), c);
        }

        Ok((styles, bindings))
    }
}

impl TryFrom<String> for Letter {
    type Error = RSError;

    fn try_from(st: String) -> Result<Letter, Self::Error> {
        let mut s = &st[..];
        let mut a = String::new();

        if let Some(lparen) = st.chars().position(|c| c == '(') {
            let rparen = match st.chars().position(|c| c == ')') {
                Some(r) => r,
                None => { return Err(RSError::ActionBindingError(st.clone())); },
            };
            a = st
                .chars()
                .skip(lparen + 1)
                .take(rparen - lparen - 1)
                .collect();
            s = &st[0..lparen];
        }

        let x = match s {
            "exit" => Letter::ExitSignal,
            "mute" => Letter::RequestMute,
            "show_output" => Letter::ChangePage(PageType::Output),
            "show_input" => Letter::ChangePage(PageType::Input),
            "show_cards" => Letter::ChangePage(PageType::Cards),
            "context_menu" => Letter::OpenContextMenu,
            "lower_volume" => {
                let a = match a.parse::<i16>() {
                    Ok(x) => x,
                    Err(_) => { return Err(RSError::ActionBindingError(st.clone())); },
                };
                Letter::RequstChangeVolume(-a)
            }
            "raise_volume" => {
                let a = match a.parse::<i16>() {
                    Ok(x) => x,
                    Err(_) => { return Err(RSError::ActionBindingError(st.clone())); },
                };
                Letter::RequstChangeVolume(a)
            }
            "up" => {
                let a = match a.parse::<u16>() {
                    Ok(x) => x,
                    Err(_) => { return Err(RSError::ActionBindingError(st.clone())); },
                };
                Letter::MoveUp(a)
            }
            "down" => {
                let a = match a.parse::<u16>() {
                    Ok(x) => x,
                    Err(_) => { return Err(RSError::ActionBindingError(st.clone())); },
                };
                Letter::MoveDown(a)
            }
            "cycle_pages_forward" => Letter::CyclePages(1),
            "cycle_pages_backward" => Letter::CyclePages(-1),
            "close_context_menu" => Letter::CloseContextMenu,
            _ => {
                return Err(RSError::ActionBindingError(st.clone()));
            }
        };
        Ok(x)
    }
}

fn find_color(s: &str) -> Option<Color> {
    if s.chars().take(1).collect::<String>() == "#" && s.len() == 7 {
        Some(Color::Rgb {
            r: u8::from_str_radix(&s[1..3], 16).expect("error in config"),
            g: u8::from_str_radix(&s[3..5], 16).expect("error in config"),
            b: u8::from_str_radix(&s[5..7], 16).expect("error in config"),
        })
    } else {
        match &s[..].parse::<Color>() {
            Ok(c) => Some(*c),
            Err(_) => None,
        }
    }
}

fn find_keycode(key: &str) -> Result<KeyEvent, RSError> {
    let mut s = String::from(key);
    let mut modifiers = KeyModifiers::empty();
    if let Some(x) = s.find("s+") {
        modifiers = modifiers | KeyModifiers::SHIFT;
        s = format!("{}{}", &s[0..x], &s[x+2..s.len()]);
    }
    if let Some(x) = s.find("c+") {
        modifiers = modifiers | KeyModifiers::CONTROL;
        s = format!("{}{}", &s[0..x], &s[x+2..s.len()]);
    }
    if let Some(x) = s.find("a+") {
        modifiers = modifiers | KeyModifiers::ALT;
        s = format!("{}{}", &s[0..x], &s[x+2..s.len()]);
    }

    let code = match &s[..] {
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
            "tab" => KeyCode::Tab,
            "backtab" => KeyCode::BackTab,
            "delete" => KeyCode::Delete,
            "insert" => KeyCode::Insert,
            "null" => KeyCode::Null,
            "esc" => KeyCode::Esc,
            _ => {
                if s.len() == 1 {
                    KeyCode::Char(s.chars().next().unwrap())
                } else {
                    return Err(RSError::KeyCodeError(String::from(key)));
                }
            }
    };

    if modifiers == KeyModifiers::SHIFT && code == KeyCode::Tab {
        Ok(KeyEvent { code: KeyCode::BackTab, modifiers: KeyModifiers::empty() })
    } else {
        Ok(KeyEvent {
            code,
            modifiers,
        })
    }
}
