mod actions;
mod colors;
mod default;
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
	pulse_audio: Option<PulseAudio>,
	bindings: MultiMap<String, String>,
	colors: LinkedHashMap<String, ConfigColor>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct PulseAudio {
	disable_live_volume: Option<bool>,
	retry_time: Option<u64>,
	rate: Option<u32>,
	frag_size: Option<u32>,
}

impl PulseAudio {
	pub fn disable_live_volume(&self) -> bool {
		self.disable_live_volume.unwrap_or(false)
	}
	pub fn retry_time(&self) -> u64 {
		self.retry_time.unwrap_or(5)
	}
	pub fn rate(&self) -> u32 {
		self.rate.unwrap_or(20)
	}
	pub fn frag_size(&self) -> u32 {
		self.frag_size.unwrap_or(48)
	}
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
