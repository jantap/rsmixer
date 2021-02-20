use super::{ToolWindowWidget, Widget};

use crate::{
    help::{self, HelpLine},
    ui::{Buffer, Rect, Style},
    RSError,
};

use crate::{scrollable, ui::Scrollable};

use itertools::Itertools;

#[derive(Clone)]
pub struct HelpWidget {
    pub window: ToolWindowWidget,
    lines: Vec<HelpLine>,
    longest_line: u16,
    min_line: u16,
    max_inner_height: u16,
    selected: usize,
}

impl Default for HelpWidget {
    fn default() -> Self {
        let lines = help::generate();
        let longest_line = lines
            .iter()
            .map(|hl| -> usize {
                hl.key_events
                    .iter()
                    .map(|kv| -> usize { kv.len() + 1 })
                    .sum::<usize>()
                    + hl.category.len()
            })
            .max()
            .unwrap_or(0);

        let min_line = lines
            .iter()
            .map(|hl| -> usize {
                hl.key_events
                    .iter()
                    .map(|kv| -> usize { kv.len() + 1 })
                    .max()
                    .unwrap_or(0)
                    + hl.category.len()
            })
            .max()
            .unwrap_or(0);

        let max_inner_height = lines
            .iter()
            .map(|hl| hl.lines_needed(longest_line as u16 + 1))
            .sum();

        Self {
            window: ToolWindowWidget::default(),
            lines,
            min_line: min_line as u16,
            longest_line: longest_line as u16,
            max_inner_height,
            selected: 0,
        }
    }
}

impl Widget for HelpWidget {
    fn resize(&mut self, area: Rect) -> Result<(), RSError> {
        if area.height < 3 || area.width < self.min_line + 2 {
            return Err(RSError::TerminalTooSmall);
        }
        self.window.padding.0 = if area.width < self.min_line + 6 { 1 } else { 3 };
        self.window.padding.1 = if area.height < 8 { 1 } else { 2 };

        self.window.inner_width = self.longest_line;
        self.window.inner_height = self.max_inner_height;

        self.window.resize(area)?;

        if self.window.area.width < 4 {
            self.window.inner_height = self
                .lines
                .iter()
                .map(|hl| hl.lines_needed(self.window.area.width - self.window.padding.0 * 2))
                .sum();

            self.window.resize(area)?;
        }

        Ok(())
    }
    fn render(&mut self, screen: &mut Buffer) -> Result<(), RSError> {
        self.window.render(screen)?;

        let inside_height = self.window.area.height - self.window.padding.1 * 2;
        let inside_width = self.window.area.width - self.window.padding.0 * 2;

        let lines = self
            .lines
            .iter()
            .map(|hl| hl.as_lines(inside_width))
            .concat();

        let (start, end) = self.visible_start_end(inside_height);

        for (i, l) in lines.iter().skip(start).take(end - start).enumerate() {
            screen.string(
                self.window.area.x + self.window.padding.0,
                self.window.area.y + self.window.padding.1 + i as u16,
                l.clone(),
                if start + i == self.selected() {
                    Style::Bold
                } else {
                    Style::Normal
                },
            );
        }

        let (first, last) = self.visible_start_end(inside_height);
        if last - first != self.len() {
            let area = Rect::new(
                self.window.area.x + self.window.padding.0,
                self.window.area.y + self.window.padding.1,
                self.window.area.width - self.window.padding.0 * 2,
                self.window.area.height - self.window.padding.1 * 2,
            );
            if first != 0 {
                screen.string(
                    area.x + area.width / 2,
                    area.y + 2,
                    "▲".to_string(),
                    Style::Normal,
                );
            }
            if last != self.len() {
                screen.string(
                    area.x + area.width / 2,
                    area.y + area.height - 2,
                    "▲".to_string(),
                    Style::Normal,
                );
            }
        }

        Ok(())
    }
}

scrollable!(
    HelpWidget,
    fn selected(&self) -> usize {
        self.selected
    },
    fn len(&self) -> usize {
        self.lines
            .iter()
            .map(|hl| hl.lines_needed(self.window.area.width - self.window.padding.0 * 2))
            .sum::<u16>() as usize
    },
    fn set_selected(&mut self, selected: usize) -> bool {
        if selected < self.len() {
            self.selected = selected;
            true
        } else {
            false
        }
    },
    fn element_height(&self, _index: usize) -> u16 {
        1
    }
);
