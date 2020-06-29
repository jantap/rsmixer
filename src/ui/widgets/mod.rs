mod block;
mod context_menu;
mod entry;
mod volume;

pub use block::BlockWidget;
pub use context_menu::ContextMenuWidget;
pub use volume::{VolumeWidget, VolumeWidgetBorder};

use super::util::Rect;
use crate::RSError;
use std::io::Write;

pub trait Widget<W: Write> {
    fn render(&mut self, area: Rect, buf: &mut W) -> Result<(), RSError>;
}
