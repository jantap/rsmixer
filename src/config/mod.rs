pub mod keys;
mod letters;
mod colors;
mod variables;

pub use variables::{VarType, Variables};

use crate::{
    Letter, RSError, Styles,
};

use std::convert::TryFrom;

use crossterm::{
    event::KeyEvent,
    style::ContentStyle,
};

use linked_hash_map::LinkedHashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RsMixerConfig {
    bindings: LinkedHashMap<String, String>,
    colors: LinkedHashMap<String, LinkedHashMap<String, String>>,
    pa_retry_time: Option<u64>,
}

impl RsMixerConfig {

    pub fn load() -> Result<Self, RSError> {
        let config: RsMixerConfig = confy::load("rsmixer")?;
        Ok(config)
    }

    pub fn interpret(&self) -> Result<(Styles, LinkedHashMap<KeyEvent, Letter>, Variables), RSError> {
        let mut bindings: LinkedHashMap<KeyEvent, Letter> = LinkedHashMap::new();

        for (k, c) in &self.bindings {
            bindings.insert(
                keys::try_string_to_keyevent(&k)?,
                Letter::try_from(c.clone())?,
            );
        }

        let mut styles: Styles = LinkedHashMap::new();

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

        Ok((styles, bindings, Variables::new(self)))
    }
}

impl std::default::Default for RsMixerConfig {
    fn default() -> Self {
        let mut bindings = LinkedHashMap::new();
        bindings.insert("q".to_string(), "exit".to_string());

        bindings.insert("j".to_string(), "down(1)".to_string());
        bindings.insert("k".to_string(), "up(1)".to_string());
        bindings.insert("down".to_string(), "down(1)".to_string());
        bindings.insert("up".to_string(), "up(1)".to_string());

        bindings.insert("left".to_string(), "lower_volume(1)".to_string());
        bindings.insert("right".to_string(), "raise_volume(1)".to_string());
        bindings.insert("h".to_string(), "lower_volume(5)".to_string());
        bindings.insert("l".to_string(), "raise_volume(5)".to_string());
        bindings.insert("shift+h".to_string(), "lower_volume(15)".to_string());
        bindings.insert("shift+l".to_string(), "raise_volume(15)".to_string());

        bindings.insert("m".to_string(), "mute".to_string());

        bindings.insert("1".to_string(), "show_output".to_string());
        bindings.insert("2".to_string(), "show_input".to_string());
        bindings.insert("3".to_string(), "show_cards".to_string());
        bindings.insert("tab".to_string(), "cycle_pages_forward".to_string());
        bindings.insert("shift+tab".to_string(), "cycle_pages_backward".to_string());

        bindings.insert("enter".to_string(), "context_menu".to_string());
        bindings.insert("esc".to_string(), "close_context_menu".to_string());

        let mut styles = LinkedHashMap::new();
        let mut c = LinkedHashMap::new();
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
            pa_retry_time: None,
        }
    }
}
