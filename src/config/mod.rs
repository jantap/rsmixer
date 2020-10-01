pub mod keys;
mod letters;
mod colors;

use crate::{
    Letter, RSError, Styles,
};

use std::{collections::HashMap, convert::TryFrom};

use crossterm::{
    event::KeyEvent,
    style::ContentStyle,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RsMixerConfig {
    bindings: HashMap<String, String>,
    colors: HashMap<String, HashMap<String, String>>,
}

impl RsMixerConfig {
    pub fn load(&self) -> Result<(Styles, HashMap<KeyEvent, Letter>), RSError> {
        let mut bindings: HashMap<KeyEvent, Letter> = HashMap::new();

        for (k, c) in &self.bindings {
            bindings.insert(
                keys::try_string_to_keyevent(&k)?,
                Letter::try_from(c.clone())?,
            );
        }

        let mut styles: Styles = HashMap::new();

        for (k, v) in &self.colors {
            let mut c = ContentStyle::new();

            if let Some(q) = v.get("fg") {
                if let Some(color) = colors::str_to_color(q) {
                    c = c.foreground(color);
                } else {
                    return Err(RSError::InvalidColor(q.clone()));
                }
            }
            if let Some(q) = v.get("bg") {
                if let Some(color) = colors::str_to_color(q) {
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
        bindings.insert("shift+tab".to_string(), "cycle_pages_backward".to_string());
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
