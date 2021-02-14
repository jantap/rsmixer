pub mod util;
pub mod widgets;

pub use util::{clean_terminal, entry_height, prepare_terminal};
use widgets::{BlockWidget, Widget};

use crate::{
    models::{PageType, RSState, Style, UIMode},
    RSError,
};

use screen_buffer_ui::{Rect, Scrollable};

use std::{collections::HashSet, io::Write};

pub type Screen = screen_buffer_ui::Screen<Style>;

pub async fn redraw<W: Write>(
    stdout: &mut W,
    ui: &mut UI,
    state: &mut RSState,
) -> Result<(), RSError> {
    if state.redraw.resize {
        ui.terminal_too_small = false;
        match ui.resize(state) {
            Err(RSError::TerminalTooSmall) => {
                ui.terminal_too_small = true;
            }
            Err(e) => {
                return Err(e);
            }
            _ => {}
        };
    }

    if ui.terminal_too_small {
        ui.screen.rect(
            Rect::new(0, 0, ui.screen.width, ui.screen.height),
            ' ',
            Style::Normal,
        );
        ui.screen
            .string(0, 0, "Terminal too small".to_string(), Style::Normal);

        ui.screen.draw_changes(stdout)?;

        return Ok(());
    }

    if state.redraw.full {
        ui.full_redraw(state)?;

        state.redraw.reset();
    }

    if state.redraw.entries {
        ui.draw_entries(state, false)?;

        state.redraw.affected_entries = HashSet::new();
    }

    if !state.redraw.affected_entries.is_empty() {
        ui.draw_entries(state, true)?;
    }

    if let Some(index) = state.redraw.peak_volume {
        if state
            .page_entries
            .visible_range(ui.screen.height)
            .any(|i| i == index)
        {
            if let Some(play) = state
                .entries
                .get_play_entry_mut(&state.page_entries.get(index).unwrap())
            {
                play.peak_volume_bar
                    .volume(play.peak)
                    .render(&mut ui.screen)?;
            }
        }
    }

    match state.ui_mode {
        UIMode::Help => state.help.render(&mut ui.screen)?,
        UIMode::ContextMenu => state.context_menu.render(&mut ui.screen)?,
        _ => {}
    };

    ui.screen.draw_changes(stdout)?;

    Ok(())
}

pub struct UI {
    pub screen: Screen,
    border: BlockWidget,
    entries_area: Rect,
    terminal_too_small: bool,
    pages_names: Vec<String>,
}

impl Default for UI {
    fn default() -> Self {
        Self {
            screen: Screen::default(),
            border: BlockWidget::default().clean_inside(true),
            entries_area: Rect::default(),
            terminal_too_small: false,
            pages_names: vec![
                PageType::Output.to_string(),
                PageType::Input.to_string(),
                PageType::Cards.to_string(),
            ],
        }
    }
}

impl UI {
    fn resize(&mut self, state: &mut RSState) -> Result<(), RSError> {
        let (x, y) = crossterm::terminal::size()?;
        self.screen.resize(x, y);

        self.border
            .resize(Rect::new(0, 0, self.screen.width, self.screen.height))?;

        self.entries_area = Rect::new(2, 2, self.screen.width - 4, self.screen.height - 4);
        let mut entry_area = self.entries_area;

        for i in state.page_entries.visible_range(self.entries_area.height) {
            let ent = match state.entries.get_mut(&state.page_entries.get(i).unwrap()) {
                Some(x) => x,
                None => {
                    continue;
                }
            };
            ent.position = state.page_entries.lvls[i];

            entry_area = entry_area.h(entry_height(ent.position));

            ent.resize(entry_area)?;

            entry_area.y += entry_height(ent.position);
        }

        state.context_menu.resize(self.entries_area)?;

        state.help.resize(self.entries_area)?;

        Ok(())
    }

    fn full_redraw(&mut self, state: &mut RSState) -> Result<(), RSError> {
        match state.ui_mode {
            UIMode::RetryIn(time) => {
                self.screen.rect(
                    Rect::new(0, 0, self.screen.width, self.screen.height),
                    ' ',
                    Style::Normal,
                );
                self.screen.string(
                    0,
                    0,
                    format!("PulseAudio disconnected. Retrying in {}...", time),
                    Style::Normal,
                );
            }
            _ => {
                self.border.render(&mut self.screen)?;

                self.draw_page_names(state);

                self.draw_entries(state, false)?;
            }
        }

        Ok(())
    }

    fn draw_page_names(&mut self, state: &mut RSState) {
        if self.screen.width as usize
            > 2 + self.pages_names.iter().map(|p| p.len()).sum::<usize>() + 6
        {
            let page: i8 = state.current_page.into();
            let page = page as usize;
            let mut length_so_far = 0;

            for (i, name) in self.pages_names.iter().enumerate() {
                self.screen.string(
                    1 + length_so_far,
                    0,
                    name.clone(),
                    if i == page { Style::Bold } else { Style::Muted },
                );
                if i != 2 {
                    self.screen.string(
                        1 + length_so_far + name.len() as u16,
                        0,
                        " / ".to_string(),
                        Style::Muted,
                    );
                    length_so_far += name.len() as u16 + 3;
                }
            }
        } else {
            self.screen
                .string(1, 0, state.current_page.to_string(), Style::Bold);
        }
    }

    fn draw_entries(&mut self, state: &mut RSState, only_affected: bool) -> Result<(), RSError> {
        for i in state.page_entries.visible_range(self.entries_area.height) {
            if only_affected && state.redraw.affected_entries.get(&i).is_none() {
                continue;
            }

            let ent = match state.entries.get_mut(&state.page_entries.get(i).unwrap()) {
                Some(x) => x,
                None => {
                    continue;
                }
            };
            ent.position = state.page_entries.lvls[i];
            ent.is_selected = state.page_entries.selected() == i;

            ent.render(&mut self.screen)?;
        }

        Ok(())
    }
}
