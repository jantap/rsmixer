pub mod util;
mod page;
mod entries;
mod common;
mod help;
pub mod widgets;

pub use util::{clean_terminal, prepare_terminal, Rect};
use common::*;
use util::terminal_too_small;
use entries::draw_entries;
pub use page::draw_page;
use help::draw_help;
use widgets::{ContextMenuWidget, VolumeWidget};

use crate::models::{RedrawType, UIMode};

pub async fn redraw<W: Write>(stdout: &mut W, state: &mut RSState) -> Result<(), RSError> {
    let (w, h) = crossterm::terminal::size()?;
    if w < 20 || h < 5 {
        return terminal_too_small(stdout).await;
    }

    if state.ui_mode == UIMode::Help && state.redraw != RedrawType::Help {
        return Ok(());
    }

    match &state.redraw {
        RedrawType::Help => {
            draw_page(
                stdout,
                state,
            )
            .await?;
            match draw_help(stdout).await {
                Err(RSError::TerminalTooSmall) => {
                    return terminal_too_small(stdout).await;
                }
                r => return r,
            };
        }
        RedrawType::Full => {
            return draw_page(
                stdout,
                state,
            )
            .await;
        }
        RedrawType::PeakVolume(ident) => {
            if ident.entry_type == EntryType::Card {
                return Ok(());
            }
            if let Some(index) = state.page_entries.iter_entries().position(|p| *p == *ident) {
                if let Some(mut area) = state.page_entries.is_entry_visible(index, state.scroll)? {
                    area.y += 2;
                    area.height = 1;
                    area.width -= 1;

                    let ent = match state.entries.get_mut(&ident) {
                        Some(x) => x,
                        None => {
                            return Ok(());
                        }
                    };

                    let area = Entry::calc_area(state.page_entries.lvls[index], area);
                    let play = ent.play_entry.as_mut().unwrap();

                    let vol = VolumeWidget::default().volume(play.peak);
                    return vol.mute(play.mute).render(area, stdout);
                }
            }
        }
        RedrawType::PartialEntries(affected) => {
            let a = affected.clone();
            return draw_entries(
                stdout,
                state,
                Some(a),
            )
            .await;
        }
        RedrawType::Entries => {
            return draw_entries(
                stdout,
                state,
                None,
            )
            .await;
        }
        RedrawType::ContextMenu => {
            let (w, h) = crossterm::terminal::size()?;
            let mut b = ContextMenuWidget::new(state.page_entries.get(state.selected).unwrap())
                .selected(state.selected_context)
                .options(state.context_options.clone());

            let a = Rect::new(2, 2, w - 4, h - 4);
            return b.render(a, stdout);
        }
        _ => {}
    };
    Ok(())
}
