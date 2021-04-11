use super::Widget;
use crate::{
	prelude::*,
	ui::{Buffer, Pixel, Rect, Style, UIError},
};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum VolumeWidgetBorder {
	Single,
	Upper,
	Lower,
	None,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct VolumeWidget {
	pub percent: f32,
	pub border: VolumeWidgetBorder,
	pub area: Rect,
	pub mute: bool,
}

impl VolumeWidget {
	pub fn default() -> Self {
		Self {
			percent: 0.0,
			border: VolumeWidgetBorder::Single,
			area: Rect::default(),
			mute: false,
		}
	}

	pub fn volume(mut self, percent: f32) -> Self {
		self.percent = percent;
		self
	}

	pub fn border(mut self, border: VolumeWidgetBorder) -> Self {
		self.border = border;
		self
	}

	pub fn mute(mut self, mute: bool) -> Self {
		self.mute = mute;
		self
	}

	pub fn set_area(mut self, area: Rect) -> Self {
		self.area = area;
		self
	}

	fn get_segments(&self) -> (u16, u16, u16) {
		let width = self.area.width;
		let third = (0.34 * (width - 2) as f32).floor() as u16;
		let last = width - 2 - third * 2;

		(third, third * 2, third * 2 + last)
	}
}

impl Widget for VolumeWidget {
	fn resize(&mut self, area: Rect) -> Result<()> {
		if area.width < 3 || area.height < 1 {
			return Err(UIError::TerminalTooSmall.into());
		}

		self.area = area;

		Ok(())
	}
	fn render(&mut self, buffer: &mut Buffer) -> Result<()> {
		self.border.render(buffer, &self.area);

		let filled = (self.percent * (self.area.width - 2) as f32).floor() as u16;

		let mut pixels: Vec<Pixel> = (0..(self.area.width - 2))
			.map(|i| Pixel {
				text: if i < filled { Some('▮') } else { Some('-') },
				style: Style::Muted,
			})
			.collect();

		if !self.mute {
			let segments = self.get_segments();

			for i in 0..segments.0 {
				pixels[i as usize].style = Style::Green;
			}
			for i in segments.0..segments.1 {
				pixels[i as usize].style = Style::Orange;
			}
			for i in segments.1..segments.2 {
				pixels[i as usize].style = Style::Red;
			}
		}

		buffer.pixels(self.area.x + 1, self.area.y, pixels);

		Ok(())
	}
}

impl VolumeWidgetBorder {
	fn render(&mut self, buffer: &mut Buffer, area: &Rect) {
		if *self == VolumeWidgetBorder::None {
			return;
		}

		let ch1 = match self {
			VolumeWidgetBorder::Single => "[",
			VolumeWidgetBorder::Upper => "┌",
			VolumeWidgetBorder::Lower => "└",
			_ => "",
		};
		let ch2 = match self {
			VolumeWidgetBorder::Single => "]",
			VolumeWidgetBorder::Upper => "┐",
			VolumeWidgetBorder::Lower => "┘",
			_ => "",
		};

		buffer.string(area.x, area.y, ch1.to_string(), Style::Normal);
		buffer.string(
			area.x + area.width - 1,
			area.y,
			ch2.to_string(),
			Style::Normal,
		);
	}
}
