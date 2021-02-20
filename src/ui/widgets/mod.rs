mod block;
mod context_menu;
mod entry;
mod help;
mod tool_window;
mod volume;

pub use block::BlockWidget;
pub use help::HelpWidget;
pub use tool_window::ToolWindowWidget;
pub use volume::{VolumeWidget, VolumeWidgetBorder};

use super::{Buffer, Rect};

use crate::RSError;

pub trait Widget {
    fn render(&mut self, screen: &mut Buffer) -> Result<(), RSError>;
    fn resize(&mut self, area: Rect) -> Result<(), RSError>;
}
