mod buffer;
mod rect;
mod scrollable;
pub mod util;
pub mod widgets;

use buffer::{Buffer, Pixel};
pub use rect::Rect;
pub use scrollable::Scrollable;
pub use util::{clean_terminal, entry_height, prepare_terminal};
use widgets::{BlockWidget, Widget};

use crate::{
    entry::EntryKind,
    models::{PageType, RSState, Style, UIMode},
    RSError,
};

use std::{collections::HashSet, io::Write};

pub async fn redraw<W: Write>(stdout: &mut W, state: &mut RSState) -> Result<(), RSError> {
    if state.redraw.resize {
        state.ui.terminal_too_small = false;
        match resize(state) {
            Err(RSError::TerminalTooSmall) => {
                state.ui.terminal_too_small = true;
            }
            Err(e) => {
                return Err(e);
            }
            _ => {}
        };
    }

    if state.ui.terminal_too_small {
        state.ui.screen.rect(
            Rect::new(0, 0, state.ui.screen.width, state.ui.screen.height),
            ' ',
            Style::Normal,
        );
        state
            .ui
            .screen
            .string(0, 0, "Terminal too small".to_string(), Style::Normal);

        state.ui.screen.draw_changes(stdout)?;

        return Ok(());
    }

    if state.redraw.full {
        full_redraw(state)?;

        state.redraw.reset();
    }

    if state.redraw.entries {
        draw_entries(state, false)?;

        state.redraw.affected_entries = HashSet::new();
    }

    if !state.redraw.affected_entries.is_empty() {
        draw_entries(state, true)?;
    }

    if let Some(index) = state.redraw.peak_volume {
        if state
            .page_entries
            .visible_range(state.ui.screen.height)
            .any(|i| i == index)
        {
            if let Some(play) = state
                .entries
                .get_play_entry_mut(&state.page_entries.get(index).unwrap())
            {
                play.peak_volume_bar
                    .volume(play.peak)
                    .render(&mut state.ui.screen)?;
            }
        }
    }

    match state.ui_mode {
        UIMode::Help => state.help.render(&mut state.ui.screen)?,
        UIMode::ContextMenu => state.context_menu.render(&mut state.ui.screen)?,
        _ => {}
    };

    state.ui.screen.draw_changes(stdout)?;

    Ok(())
}

pub struct UI {
    pub screen: Buffer,
    border: BlockWidget,
    pub entries_area: Rect,
    terminal_too_small: bool,
    pub pages_names: Vec<String>,
}

impl Default for UI {
    fn default() -> Self {
        Self {
            screen: Buffer::default(),
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

fn resize(state: &mut RSState) -> Result<(), RSError> {
    let (x, y) = crossterm::terminal::size()?;
    state.ui.screen.resize(x, y);

    state.ui.border.resize(Rect::new(
        0,
        0,
        state.ui.screen.width,
        state.ui.screen.height,
    ))?;

    state.ui.entries_area = Rect::new(2, 2, state.ui.screen.width - 4, state.ui.screen.height - 4);
    let mut entry_area = state.ui.entries_area;

    for i in state
        .page_entries
        .visible_range(state.ui.entries_area.height)
    {
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

    state.context_menu.resize(state.ui.entries_area)?;

    state.help.resize(state.ui.entries_area)?;

    Ok(())
}

fn full_redraw(state: &mut RSState) -> Result<(), RSError> {
    match state.ui_mode {
        UIMode::RetryIn(time) => {
            state.ui.screen.rect(
                Rect::new(0, 0, state.ui.screen.width, state.ui.screen.height),
                ' ',
                Style::Normal,
            );
            state.ui.screen.string(
                0,
                0,
                format!("PulseAudio disconnected. Retrying in {}...", time),
                Style::Normal,
            );
        }
        _ => {
            state.ui.border.render(&mut state.ui.screen)?;

            draw_page_names(state);

            draw_entries(state, false)?;
        }
    }

    Ok(())
}

fn draw_page_names(state: &mut RSState) {
    if state.ui.screen.width as usize
        > 2 + state.ui.pages_names.iter().map(|p| p.len()).sum::<usize>() + 6
    {
        let page: i8 = state.current_page.into();
        let page = page as usize;
        let mut length_so_far = 0;

        for (i, name) in state.ui.pages_names.iter().enumerate() {
            state.ui.screen.string(
                1 + length_so_far,
                0,
                name.clone(),
                if i == page { Style::Bold } else { Style::Muted },
            );
            if i != 2 {
                state.ui.screen.string(
                    1 + length_so_far + name.len() as u16,
                    0,
                    " / ".to_string(),
                    Style::Muted,
                );
                length_so_far += name.len() as u16 + 3;
            }
        }
    } else {
        state
            .ui
            .screen
            .string(1, 0, state.current_page.to_string(), Style::Bold);
    }
}

fn draw_entries(state: &mut RSState, only_affected: bool) -> Result<(), RSError> {
    for i in state
        .page_entries
        .visible_range(state.ui.entries_area.height)
    {
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

        ent.render(&mut state.ui.screen)?;

        if i + 1 == state.page_entries.len() {
            let area = match &ent.entry_kind {
                EntryKind::CardEntry(card) => card.area,
                EntryKind::PlayEntry(play) => play.area,
            };
            let bottom = Rect::new(
                area.x,
                area.y + area.height,
                area.width,
                state.ui.entries_area.height - area.y - area.height,
            );

            state.ui.screen.rect(bottom, ' ', Style::Normal);
        }
    }

    Ok(())
}
