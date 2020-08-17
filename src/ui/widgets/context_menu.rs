use super::{super::models::ContextMenuOption, BlockWidget, Widget};

use crate::{
    draw_at,
    entry::EntryIdentifier,
    ui::util::{get_style, Rect},
    RSError,
};

use std::{cmp::max, io::Write};

use crossterm::execute;

#[derive(Clone)]
pub struct ContextMenuWidget {
    identifier: EntryIdentifier,
    options: Vec<ContextMenuOption>,
    scrolling: bool,
    selected: usize,
}

impl ContextMenuWidget {
    pub fn new(identifier: EntryIdentifier) -> Self {
        Self {
            identifier,
            options: Vec::new(),
            scrolling: false,
            selected: 0,
        }
    }

    pub fn selected(mut self, selected: usize) -> Self {
        self.selected = selected;
        self
    }

    pub fn options(mut self, options: Vec<ContextMenuOption>) -> Self {
        self.options = options;
        self
    }
}

impl<W: Write> Widget<W> for ContextMenuWidget {
    fn render(&mut self, mut area: Rect, buf: &mut W) -> Result<(), RSError> {
        let recommended_h = self.options.len() + 6;
        if recommended_h < area.height as usize {
            self.scrolling = false;
            area.height = recommended_h as u16;
        } else {
            self.scrolling = true;
        }

        let mut longest_word = 0;
        self.options.iter().for_each(|o| {
            longest_word = max(longest_word, String::from(o.clone()).len());
        });

        let target_w = max(40, longest_word + 4) as u16;

        if area.width > target_w {
            area.x += (area.width - target_w) / 2;
            area.width = target_w;
        }

        let mut b = BlockWidget::default().clean_inside(true);
        b.render(area, buf)?;

        let mut starty = area.y + 3;

        for (i, o) in self.options.iter().enumerate() {
            let s: String = o.clone().into();
            let startx = area.x + area.width / 2 - s.len() as u16 / 2;
            draw_at!(
                buf,
                s,
                startx,
                starty,
                if self.selected == i {
                    get_style("inverted")
                } else {
                    get_style("normal")
                }
            );

            starty += 1;
        }

        buf.flush()?;

        Ok(())
    }
}
