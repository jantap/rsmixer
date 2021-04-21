use super::Widget;
use crate::{
	models::Style,
	prelude::*,
	repeat,
	ui::{Buffer, Pixels, Rect, UIError},
};

#[derive(Clone)]
pub struct BlockWidget {
	pub area: Rect,
	pub title: Option<String>,
	pub title_pixels: Option<Pixels>,
	pub clean_inside: bool,
}

impl Default for BlockWidget {
	fn default() -> Self {
		Self {
			title: None,
			title_pixels: None,
			clean_inside: false,
			area: Rect::default(),
		}
	}
}

impl BlockWidget {
	pub fn clean_inside(mut self, clean: bool) -> Self {
		self.clean_inside = clean;
		self
	}
}

impl Widget for BlockWidget {
	fn resize(&mut self, area: Rect) -> Result<()> {
		if area.width < 2 || area.height < 2 {
			return Err(UIError::TerminalTooSmall.into());
		}

		self.area = area;

		Ok(())
	}
	fn render(&mut self, buffer: &mut Buffer) -> Result<()> {
		let mut top_border = Pixels::default().string(
			Style::Normal,
			&format!("┌{}┐", repeat!("─", self.area.width - 2)),
		);

		if let Some(title) = &self.title {
			for (i, c) in title.chars().enumerate() {
				if let Some(pixel) = top_border.get_mut(i + 1) {
					pixel.text = Some(c);
				} else {
					break;
				}
			}
		} else if let Some(title) = &mut self.title_pixels {
			for (i, p) in title.iter_mut().enumerate() {
				if let Some(pixel) = top_border.get_mut(i + 1) {
					*pixel = *p;
				} else {
					break;
				}
			}
		}
		buffer.pixels(self.area.x, self.area.y, &top_border);

		if self.clean_inside {
			let mut middle = Pixels::default().next(Style::Normal, '│');
			for _ in 0..(self.area.width - 2) {
				middle = middle.next(Style::Normal, ' ');
			}
			middle = middle.next(Style::Normal, '│');

			for i in 1..(self.area.height - 1) {
				buffer.pixels(self.area.x, self.area.y + i, &middle);
			}
		} else {
			for i in 1..(self.area.height - 1) {
				buffer.string(self.area.x, self.area.y + i, "│".to_string(), Style::Normal);
				buffer.string(
					self.area.x + self.area.width - 1,
					self.area.y + i,
					"│".to_string(),
					Style::Normal,
				);
			}
		}

		let bottom_border = format!("└{}┘", repeat!("─", self.area.width - 2));

		buffer.string(
			self.area.x,
			self.area.y + self.area.height - 1,
			bottom_border,
			Style::Normal,
		);

		Ok(())
	}
}
