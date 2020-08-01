use crate::{ui::PageType, Letter, Styles};
use crossterm::event::KeyCode;
use crossterm::style::Color;
use crossterm::style::ContentStyle;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
        bindings.insert("Enter".to_string(), "context_menu".to_string());
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
        styles.insert("green".to_string(), c.clone());
        Self {
            bindings,
            colors: styles,
        }
    }
}

impl RsMixerConfig {
    pub fn load(&self) -> (Styles, HashMap<KeyCode, Letter>) {
        let mut bindings: HashMap<KeyCode, Letter> = HashMap::new();

        for (k, c) in &self.bindings {
            bindings.insert(find_keycode(&k), c.clone().into());
        }

        let mut styles: Styles = HashMap::new();

        for (k, v) in &self.colors {
            let mut c = ContentStyle::new();
            if let Some(q) = v.get("fg") {
                if let Some(color) = find_color(q) {
                    c = c.foreground(color);
                }
            }
            if let Some(q) = v.get("bg") {
                if let Some(color) = find_color(q) {
                    c = c.background(color);
                }
            }
            styles.insert(k.clone(), c);
        }

        (styles, bindings)
    }
}

impl From<String> for Letter {
    fn from(st: String) -> Letter {
        let mut s = &st[..];
        let mut a = String::new();

        if let Some(lparen) = st.chars().position(|c| c == '(') {
            let rparen = st.chars().position(|c| c == ')').unwrap();
            a = st
                .chars()
                .skip(lparen + 1)
                .take(rparen - lparen - 1)
                .collect();
            s = &st[0..lparen];
        }

        match s {
            "exit" => Letter::ExitSignal,
            "mute" => Letter::RequestMute,
            "show_output" => Letter::ChangePage(PageType::Output),
            "show_input" => Letter::ChangePage(PageType::Input),
            "show_cards" => Letter::ChangePage(PageType::Cards),
            "context_menu" => Letter::OpenContextMenu,
            "lower_volume" => Letter::RequstChangeVolume(-(a.parse::<i16>().unwrap())),
            "raise_volume" => Letter::RequstChangeVolume(a.parse().unwrap()),
            "up" => Letter::MoveUp(a.parse().unwrap()),
            "down" => Letter::MoveDown(a.parse().unwrap()),
            _ => Letter::ExitSignal,
        }
    }
}

fn find_color(s: &String) -> Option<Color> {
    if s.chars().take(1).collect::<String>() == "#" && s.len() == 7 {
        return Some(Color::Rgb {
            r: u8::from_str_radix(&s[1..3], 16).expect("error in config"),
            g: u8::from_str_radix(&s[3..5], 16).expect("error in config"),
            b: u8::from_str_radix(&s[5..7], 16).expect("error in config"),
        });
    } else {
        return match &s[..].parse::<Color>() {
            Ok(c) => Some(c.clone()),
            Err(_) => None,
        };
    }
}

fn find_keycode(s: &String) -> KeyCode {
    match &s[..] {
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
        _ => KeyCode::Char(s.chars().next().unwrap()),
    }
}
