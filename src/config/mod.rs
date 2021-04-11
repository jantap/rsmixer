mod actions;
mod colors;
mod errors;
pub mod keys_mouse;
mod variables;

use std::{collections::HashMap, convert::TryFrom};

use crossterm::style::{Attribute, ContentStyle};
pub use errors::ConfigError;
use linked_hash_map::LinkedHashMap;
use semver::Version;
use serde::{Deserialize, Serialize};
pub use variables::Variables;

use crate::{
	models::{InputEvent, UserAction},
	multimap::MultiMap,
	prelude::*,
	Styles, VERSION,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct RsMixerConfig {
	version: Option<String>,
	pa_retry_time: Option<u64>,
	bindings: MultiMap<String, String>,
	colors: LinkedHashMap<String, ConfigColor>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConfigColor {
	fg: Option<String>,
	bg: Option<String>,
	attributes: Option<Vec<String>>,
}

impl RsMixerConfig {
	pub fn load() -> Result<Self> {
		let config: RsMixerConfig = confy::load("rsmixer")?;
		Ok(config)
	}

	pub fn interpret(&mut self) -> Result<(Styles, MultiMap<InputEvent, UserAction>, Variables)> {
		self.compatibility_layer()?;

		let bindings = self.bindings()?;

		let mut styles: Styles = HashMap::new();

		for (k, v) in &self.colors {
			let mut c = ContentStyle::new();

			if let Some(q) = &v.fg {
				if let Some(color) = colors::str_to_color(q) {
					c = c.foreground(color);
				} else {
					return Err(ConfigError::InvalidColor(q.clone()))
						.context("while parsing config file");
				}
			}
			if let Some(q) = &v.bg {
				if let Some(color) = colors::str_to_color(q) {
					c = c.background(color);
				} else {
					return Err(ConfigError::InvalidColor(q.clone()))
						.context("while parsing config file");
				}
			}
			if let Some(attrs) = &v.attributes {
				for attr in attrs {
					match &attr[..] {
						"bold" => {
							c = c.attribute(Attribute::Bold);
						}
						"underlined" => {
							c = c.attribute(Attribute::Underlined);
						}
						"italic" => {
							c = c.attribute(Attribute::Italic);
						}
						"dim" => {
							c = c.attribute(Attribute::Dim);
						}
						_ => {}
					};
				}
			}
			styles.insert(k.into(), c);
		}

		self.version = Some(String::from(VERSION));

		confy::store("rsmixer", self.clone())?;

		Ok((styles, bindings, Variables::new(self)))
	}

	fn bindings(&self) -> Result<MultiMap<InputEvent, UserAction>> {
		let mut bindings: MultiMap<InputEvent, UserAction> = MultiMap::new();

		for (k, cs) in self.bindings.iter_vecs() {
			for c in cs {
				bindings.insert(
					keys_mouse::try_string_to_event(&k)?,
					UserAction::try_from(c.clone())?,
				);
			}
		}

		Ok(bindings)
	}

	fn compatibility_layer(&mut self) -> Result<()> {
		let current_ver = Version::parse(VERSION)?;

		let config_ver = match &self.version {
			Some(v) => v.clone(),
			None => "0.0.0".to_string(),
		};
		let config_ver = Version::parse(&config_ver)?;

		if config_ver >= current_ver {
			return Ok(());
		}

		if config_ver == Version::parse("0.3.0").unwrap() {
			if let Some(c) = self.colors.get(&"normal".to_string()) {
				let mut c = c.clone();
				c.attributes = Some(vec!["bold".to_string()]);
				self.colors.insert("bold".to_string(), c);
			}
			return Ok(());
		}

		let mut parsed: MultiMap<InputEvent, (UserAction, String)> = MultiMap::new();

		for (k, cs) in self.bindings.iter_vecs() {
			for c in cs {
				parsed.insert(
					keys_mouse::try_string_to_event(&k)?,
					(UserAction::try_from(c.clone())?, k.clone()),
				);
			}
		}

		if parsed
			.iter()
			.find(|(_, v)| (**v).0 == UserAction::Confirm)
			.is_none()
		{
			if let Some((_, (_, k))) = parsed
				.iter()
				.find(|(_, v)| (**v).0 == UserAction::OpenContextMenu(None))
			{
				self.bindings
					.insert(k.clone(), UserAction::Confirm.to_string());
			}
		}

		Ok(())
	}
}

impl std::default::Default for RsMixerConfig {
	fn default() -> Self {
		let mut bindings = MultiMap::new();
		bindings.insert("q".to_string(), "exit".to_string());
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

		bindings.insert("m".to_string(), "mute".to_string());

		bindings.insert("1".to_string(), "show_output".to_string());
		bindings.insert("2".to_string(), "show_input".to_string());
		bindings.insert("3".to_string(), "show_cards".to_string());
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
				bg: Some("black".to_string()),
				attributes: None,
			},
		);
		c.insert(
			"bold".to_string(),
			ConfigColor {
				fg: Some("white".to_string()),
				bg: Some("black".to_string()),
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
				bg: Some("black".to_string()),
				attributes: None,
			},
		);
		c.insert(
			"red".to_string(),
			ConfigColor {
				fg: Some("red".to_string()),
				bg: Some("black".to_string()),
				attributes: None,
			},
		);
		c.insert(
			"orange".to_string(),
			ConfigColor {
				fg: Some("yellow".to_string()),
				bg: Some("black".to_string()),
				attributes: None,
			},
		);
		c.insert(
			"green".to_string(),
			ConfigColor {
				fg: Some("green".to_string()),
				bg: Some("black".to_string()),
				attributes: None,
			},
		);

		Self {
			version: Some(String::from(VERSION)),
			pa_retry_time: None,
			bindings,
			colors: c,
		}
	}
}
