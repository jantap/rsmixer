use std::cmp::{max, min};

use super::{BlockWidget, Widget};
use crate::{
	ui::{Buffer, Rect},
	RsError,
};

#[derive(Clone)]

pub struct ToolWindowWidget {
	pub area: Rect,
	border: BlockWidget,
	pub inner_width: u16,
	pub inner_height: u16,
	pub padding: (u16, u16),
}

impl Default for ToolWindowWidget {
	fn default() -> Self {
		Self {
			area: Rect::default(),
			border: BlockWidget::default().clean_inside(true),
			inner_width: 0,
			inner_height: 0,
			padding: (2, 3),
		}
	}
}

impl Widget for ToolWindowWidget {
	fn resize(&mut self, mut area: Rect) -> Result<(), RsError> {
		let target_h = min(self.inner_height + self.padding.1 * 2, area.height);

		let target_w = min(
			max(40, self.inner_width + self.padding.0 * 2) as u16,
			area.width,
		);

		if area.width > target_w {
			area.x += (area.width - target_w) / 2;
		}
		if area.height > target_h {
			area.y += (area.height - target_h) / 2;
		}

		area.width = target_w;
		area.height = target_h;

		self.area = area;
		self.border.resize(area)?;

		Ok(())
	}
	fn render(&mut self, buffer: &mut Buffer) -> Result<(), RsError> {
		self.border.render(buffer)?;

		Ok(())
	}
}
