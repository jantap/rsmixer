mod block;
mod context_menu;
mod entry;
mod help;
mod tool_window;
mod volume;
mod warning_text;

pub use block::BlockWidget;
pub use help::HelpWidget;
pub use tool_window::ToolWindowWidget;
pub use volume::{VolumeWidget, VolumeWidgetBorder};
pub use warning_text::WarningTextWidget;

use super::{Buffer, Rect};
use crate::RsError;

pub trait Widget {
	fn render(&mut self, buffer: &mut Buffer) -> Result<(), RsError>;
	fn resize(&mut self, area: Rect) -> Result<(), RsError>;
}
