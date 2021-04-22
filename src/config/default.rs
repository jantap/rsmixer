use super::{ConfigColor, RsMixerConfig};
use linked_hash_map::LinkedHashMap;
use crate::{VERSION, multimap::MultiMap};

impl std::default::Default for RsMixerConfig {
	fn default() -> Self {
		let mut bindings = MultiMap::new();
		bindings.insert("q".to_string(), "exit".to_string());
		bindings.insert("ctrl+c".to_string(), "exit".to_string());
		bindings.insert("?".to_string(), "help".to_string());

		bindings.insert("j".to_string(), "down(1)".to_string());
		bindings.insert("k".to_string(), "up(1)".to_string());
		bindings.insert("h".to_string(), "left".to_string());
		bindings.insert("l".to_string(), "right".to_string());
		bindings.insert("down".to_string(), "down(1)".to_string());
		bindings.insert("up".to_string(), "up(1)".to_string());

		bindings.insert("left".to_string(), "lower_volume(1)".to_string());
		bindings.insert("right".to_string(), "raise_volume(1)".to_string());
		bindings.insert("h".to_string(), "lower_volume(5)".to_string());
		bindings.insert("l".to_string(), "raise_volume(5)".to_string());
		bindings.insert("shift+h".to_string(), "lower_volume(15)".to_string());
		bindings.insert("shift+l".to_string(), "raise_volume(15)".to_string());
		bindings.insert("scroll_down".to_string(), "lower_volume(5)".to_string());
		bindings.insert("scroll_up".to_string(), "raise_volume(5)".to_string());

		bindings.insert("m".to_string(), "mute".to_string());
		bindings.insert("mouse_middle".to_string(), "mute".to_string());
		bindings.insert("mouse_right".to_string(), "mute".to_string());

		bindings.insert("1".to_string(), "show_output".to_string());
		bindings.insert("2".to_string(), "show_input".to_string());
		bindings.insert("3".to_string(), "show_cards".to_string());
		bindings.insert("F1".to_string(), "show_output".to_string());
		bindings.insert("F2".to_string(), "show_input".to_string());
		bindings.insert("F3".to_string(), "show_cards".to_string());
		bindings.insert("tab".to_string(), "cycle_pages_forward".to_string());
		bindings.insert("shift+tab".to_string(), "cycle_pages_backward".to_string());

		bindings.insert("enter".to_string(), "context_menu".to_string());
		bindings.insert("enter".to_string(), "confirm".to_string());
		bindings.insert("esc".to_string(), "close_context_menu".to_string());
		bindings.insert("q".to_string(), "close_context_menu".to_string());

		let mut c = LinkedHashMap::new();
		c.insert(
			"normal".to_string(),
			ConfigColor {
				fg: Some("white".to_string()),
                bg: None,
				attributes: None,
			},
		);
		c.insert(
			"bold".to_string(),
			ConfigColor {
				fg: Some("white".to_string()),
                bg: None,
				attributes: Some(vec!["bold".to_string()]),
			},
		);
		c.insert(
			"inverted".to_string(),
			ConfigColor {
				fg: Some("black".to_string()),
				bg: Some("white".to_string()),
				attributes: None,
			},
		);
		c.insert(
			"muted".to_string(),
			ConfigColor {
				fg: Some("grey".to_string()),
                bg: None,
				attributes: None,
			},
		);
		c.insert(
			"red".to_string(),
			ConfigColor {
				fg: Some("red".to_string()),
                bg: None,
				attributes: None,
			},
		);
		c.insert(
			"orange".to_string(),
			ConfigColor {
				fg: Some("yellow".to_string()),
                bg: None,
				attributes: None,
			},
		);
		c.insert(
			"green".to_string(),
			ConfigColor {
				fg: Some("green".to_string()),
                bg: None,
				attributes: None,
			},
		);

		Self {
			version: Some(String::from(VERSION)),
			pulse_audio: None,
			bindings,
			colors: c,
		}
	}
}
