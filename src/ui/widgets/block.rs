use super::Widget;
use crate::{
    models::Style,
    repeat,
    ui::{Buffer, Rect},
    RsError,
};

#[derive(Clone)]
pub struct BlockWidget {
    pub area: Rect,
    pub title: String,
    pub title_len: u16,
    pub clean_inside: bool,
}

impl Default for BlockWidget {
    fn default() -> Self {
        Self {
            title: String::from(""),
            title_len: 0,
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
    fn resize(&mut self, area: Rect) -> Result<(), RsError> {
        if area.width < 2 || area.height < 2 {
            return Err(RsError::TerminalTooSmall);
        }

        self.area = area;

        Ok(())
    }
    fn render(&mut self, buffer: &mut Buffer) -> Result<(), RsError> {
        let top_border = format!(
            "┌{}",
            if self.title.len() < self.area.width as usize - 2 {
                &self.title[..]
            } else {
                &self.title[0..(self.area.width as usize - 2)]
            }
        );

        let top_border = format!(
            "{}{}┐",
            top_border,
            repeat!("─", self.area.width + 1 - top_border.len() as u16)
        );

        let bottom_border = format!("└{}┘", repeat!("─", self.area.width - 2));

        buffer.string(self.area.x, self.area.y, top_border, Style::Normal);
        buffer.string(
            self.area.x,
            self.area.y + self.area.height - 1,
            bottom_border,
            Style::Normal,
        );

        for i in 1..(self.area.height - 1) {
            buffer.string(self.area.x, self.area.y + i, "│".to_string(), Style::Normal);
            buffer.string(
                self.area.x + self.area.width - 1,
                self.area.y + i,
                "│".to_string(),
                Style::Normal,
            );
        }

        if self.clean_inside {
            buffer.rect(
                Rect::new(
                    self.area.x + 1,
                    self.area.y + 1,
                    self.area.width - 2,
                    self.area.height - 2,
                ),
                ' ',
                Style::Normal,
            );
        }

        Ok(())
    }
}
