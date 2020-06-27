use crate::{
    draw_at,
    draw_range, repeat_string,
    ui::{
        util::{get_style, Rect},
        Widget,
    },
    Result,
};
use super::super::EntryIdentifier;
use super::BlockWidget;

use std::io::Write;

use crossterm::{cursor::MoveTo, execute};

#[derive(Clone)]
pub struct ContextMenuWidget {
    identifier: EntryIdentifier,
    options: Vec<&'static str>,
    scrolling: bool,
    selected: usize,
}

impl ContextMenuWidget {
    pub fn new(identifier: EntryIdentifier) -> Self {
        Self {
            identifier,
            options: vec!["Move", "Kill"],
            scrolling: false,
            selected: 0,
        }
    }

    pub fn selected(mut self, selected: usize) -> Self {
        self.selected = selected;
        self
    }
}

impl<W: Write> Widget<W> for ContextMenuWidget {
    fn render(mut self, mut area: Rect, buf: &mut W) -> Result<()> {
        let recommended_h = self.options.len() + 6;
        if recommended_h < area.height as usize {
            self.scrolling = false;
            area.height = recommended_h as u16;
        } else {
            self.scrolling = true;
        }

        if area.width > 40 {
            area.x += (area.width - 40)/2;
            area.width = 40;
        }

        let b = BlockWidget::default().clean_inside(true);
        b.render(area, buf)?;

        let mut starty = area.y + 3;

        for (i, o) in self.options.iter().enumerate() {
            let startx = area.x + area.width / 2 - o.len() as u16 / 2;
            draw_at!(buf, o, startx, starty,
                    if self.selected == i { get_style("inverted") }else {get_style("normal") });

            starty += 1;
        }

        buf.flush()?;

        Ok(())
    }
}
