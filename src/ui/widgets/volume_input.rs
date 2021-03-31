use super::{BlockWidget, Widget};
use crate::{
	models::Style,
	ui::{Buffer, Rect},
	RsError,
};

#[derive(Clone)]
pub struct VolumeInputWidget {
	pub value: String,
	pub cursor: u8,
	pub window: BlockWidget,
}

impl Default for VolumeInputWidget {
	fn default() -> Self {
		Self {
			value: "".to_string(),
			cursor: 0,
			window: BlockWidget::default(),
		}
	}
}

impl Widget for VolumeInputWidget {
	fn resize(&mut self, area: Rect) -> Result<(), RsError> {
		self.window.resize(area)?;
		Ok(())
	}

	fn render(&mut self, buffer: &mut Buffer) -> Result<(), RsError> {
		self.window.render(buffer)?;

		buffer.string(
			self.window.area.x + 1,
			self.window.area.y + 1,
			format!("{}", self.value),
			Style::Normal,
		);

		Ok(())
	}
}
