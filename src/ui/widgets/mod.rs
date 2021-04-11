mod block;
mod context_menu;
mod entry;
mod help;
mod tool_window;
mod volume;
mod volume_input;
mod warning_text;

pub use block::BlockWidget;
pub use help::HelpWidget;
pub use tool_window::ToolWindowWidget;
pub use volume::{VolumeWidget, VolumeWidgetBorder};
pub use volume_input::VolumeInputWidget;
pub use warning_text::WarningTextWidget;

use super::{Buffer, Rect};
use crate::prelude::*;

pub trait Widget {
	fn render(&mut self, buffer: &mut Buffer) -> Result<()>;
	fn resize(&mut self, area: Rect) -> Result<()>;
}
