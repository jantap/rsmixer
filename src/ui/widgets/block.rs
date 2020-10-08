use super::Widget;

use crate::{
    draw_range, repeat_string,
    ui::util::{get_style, Rect},
    RSError,
};

use std::io::Write;

use crossterm::{cursor::MoveTo, execute};

#[derive(Clone)]
pub struct BlockWidget {
    pub title: String,
    pub title_len: u16,
    pub clean_inside: bool,
}

impl BlockWidget {
    pub fn default() -> Self {
        Self {
            title: String::from(""),
            title_len: 0,
            clean_inside: false,
        }
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    pub fn title_len(mut self, len: u16) -> Self {
        self.title_len = len;
        self
    }

    pub fn clean_inside(mut self, clean: bool) -> Self {
        self.clean_inside = clean;
        self
    }
}

impl<W: Write> Widget<W> for BlockWidget {
    fn render(&mut self, area: Rect, buf: &mut W) -> Result<(), RSError> {
        if area.width < 2 || area.height < 2 {
            return Err(RSError::TerminalTooSmall);
        }

        let l = if self.title_len > 0 { self.title_len } else { self.title.len() as u16 };

        let top = if area.width < 2 + l {
            (&self.title[0..area.width as usize - 2]).to_string()
        } else {
            format!(
                "{}{}",
                self.title,
                get_style("normal").clone().apply(repeat_string!("─", area.width - 2 - l))
            )
        };

        let top = get_style("normal").apply(format!("┌{}┐", top,));
        let bot = get_style("normal").apply(format!("└{}┘", repeat_string!("─", area.width - 2)));
        execute!(buf, MoveTo(area.x, area.y))?;
        write!(buf, "{}", top)?;
        execute!(buf, MoveTo(area.x, area.y + area.height - 1))?;
        write!(buf, "{}", bot)?;

        if self.clean_inside {
            let sides =
                get_style("normal").apply(format!("│{}│", repeat_string!(" ", area.width - 2)));
            for i in 0..area.height - 2 {
                execute!(buf, MoveTo(area.x, i + 1 + area.y))?;
                write!(buf, "{}", sides.clone())?;
            }
        } else {
            draw_range!(
                buf,
                "│",
                area.x..area.x + 1,
                area.y..area.y + area.height - 1,
                get_style("normal")
            );
            draw_range!(
                buf,
                "│",
                area.y + area.width - 1..area.y + area.width,
                area.y..area.y + area.height - 1,
                get_style("normal")
            );
        }

        buf.flush()?;

        Ok(())
    }
}
