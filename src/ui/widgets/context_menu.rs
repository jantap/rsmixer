use std::cmp::max;

use super::Widget;
use crate::{
	models::ContextMenu,
	ui::{Buffer, Rect, Scrollable, Style, UIError},
    prelude::*,
};

impl Widget for ContextMenu {
	fn resize(&mut self, area: Rect) -> Result<()> {
		let mut longest_word = 0;
		self.options.iter().for_each(|o| {
			longest_word = max(longest_word, String::from(o.clone()).len());
		});

		if area.height < 3 || area.width < 4 {
			return Err(UIError::TerminalTooSmall.into());
		}
		self.tool_window.padding.0 = if area.width < longest_word as u16 + 6 {
			1
		} else {
			3
		};
		self.tool_window.padding.1 = if area.height < 8 { 1 } else { 2 };

		self.tool_window.inner_width = longest_word as u16;
		self.tool_window.inner_height = self.options.len() as u16;

		self.tool_window.resize(area)?;

		self.area = Rect::new(
			self.tool_window.area.x + self.tool_window.padding.0,
			self.tool_window.area.y + self.tool_window.padding.1,
			self.tool_window.area.width - self.tool_window.padding.0 * 2,
			self.tool_window.area.height - self.tool_window.padding.1 * 2,
		);

		Ok(())
	}
	fn render(&mut self, buffer: &mut Buffer) -> Result<()> {
		self.tool_window.render(buffer)?;

		for (y, i) in self.visible_range(self.area.height).enumerate() {
			let text: String = self.options[i].clone().into();

			let text: String = text
				.chars()
				.skip(self.horizontal_scroll * self.area.width as usize)
				.take(self.area.width as usize)
				.collect();

			let text_x = if self.horizontal_scroll > 0 {
				self.area.x
			} else {
				self.area.x + self.area.width / 2 - text.len() as u16 / 2
			};
			buffer.string(
				text_x,
				self.area.y + y as u16,
				text,
				if self.selected() == i {
					Style::Inverted
				} else {
					Style::Normal
				},
			);
		}

		let (first, last) = self.visible_start_end(self.area.height);
		if last - first != self.len() {
			if first != 0 {
				buffer.string(
					self.area.x + self.area.width / 2,
					self.area.y - 1,
					"▲".to_string(),
					Style::Normal,
				);
			}
			if last != self.len() {
				buffer.string(
					self.area.x + self.area.width / 2,
					self.area.y + self.area.height,
					"▼".to_string(),
					Style::Normal,
				);
			}
		}

		let max_horizontal_scroll = self.max_horizontal_scroll();
		if self.horizontal_scroll < max_horizontal_scroll {
			buffer.string(
				self.area.x + self.area.width,
				self.area.y + self.area.height / 2,
				"▶".to_string(),
				Style::Normal,
			);
		}
		if self.horizontal_scroll > 0 {
			buffer.string(
				self.area.x - 1,
				self.area.y + self.area.height / 2,
				"◀".to_string(),
				Style::Normal,
			);
		}

		Ok(())
	}
}
