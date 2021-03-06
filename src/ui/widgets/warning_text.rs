use super::Widget;

use crate::{models::Style, ui::Buffer, RsError};

use crate::ui::Rect;

#[derive(Clone)]
pub struct WarningTextWidget {
    pub text: String,
}

impl Default for WarningTextWidget {
    fn default() -> Self {
        Self {
            text: String::from(""),
        }
    }
}

impl Widget for WarningTextWidget {
    fn resize(&mut self, _area: Rect) -> Result<(), RsError> {
        Ok(())
    }
    fn render(&mut self, buffer: &mut Buffer) -> Result<(), RsError> {
        buffer.rect(
            Rect::new(0, 0, buffer.width, buffer.height),
            ' ',
            Style::Normal,
        );
        buffer.string(0, 0, self.text.clone(), Style::Normal);

        Ok(())
    }
}
