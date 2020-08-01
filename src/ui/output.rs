use super::action_handlers::*;
pub use super::widgets::VolumeWidget;
use crate::entry::{Entries, Entry, EntryIdentifier, EntryType};
use crate::ui::draw::{draw_page, redraw};
use crate::ui::models::{ContextMenuOption, PageEntries};
use crate::{RSError, DISPATCH};
use std::collections::HashMap;

use super::util::parent_child_types;
use crate::Letter;

pub use super::util::PageType;
use std::io::Write;

use tokio::stream::StreamExt;
use tokio::sync::broadcast::Receiver;

use std::io;

use crossterm::{cursor::Hide, execute};

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum RedrawType {
    Full,
    Entries,
    PeakVolume(EntryIdentifier),
    ContextMenu,
    None,
    Exit,
}
impl Eq for RedrawType {}

impl From<RedrawType> for u32 {
    fn from(redraw: RedrawType) -> u32 {
        match redraw {
            RedrawType::Full => 1000,
            RedrawType::Entries => 500,
            RedrawType::ContextMenu => 500,
            RedrawType::PeakVolume(_) => 100,
            RedrawType::None => 1,
            RedrawType::Exit => 10000,
        }
    }
}
impl std::cmp::PartialOrd for RedrawType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for RedrawType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let a = u32::from(*self);
        let b = u32::from(*other);

        if a > b {
            std::cmp::Ordering::Greater
        } else if b > a {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

impl RedrawType {
    fn take_bigger(&mut self, mut other: Self) {
        if self.cmp(&&mut other) == std::cmp::Ordering::Less {
            *self = other;
        }
    }
}

#[derive(PartialEq)]
pub enum UIMode {
    Normal,
    ContextMenu,
}

pub struct UIState {
    pub current_page: PageType,
    pub entries: Entries,
    pub page_entries: PageEntries,
    pub selected: usize,
    pub selected_context: usize,
    pub context_options: Vec<ContextMenuOption>,
    pub scroll: usize,
    pub redraw: RedrawType,
    pub ui_mode: UIMode,
}

pub async fn ui_loop(mut rx: Receiver<Letter>) -> Result<(), RSError> {
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;

    let mut state = UIState {
        current_page: PageType::Output,
        entries: Entries::new(),
        page_entries: PageEntries::new(),
        selected: 0,
        selected_context: 0,
        context_options: Vec::new(),
        scroll: 0,
        redraw: RedrawType::None,
        ui_mode: UIMode::Normal,
    };
    execute!(stdout, Hide)?;
    draw_page(
        &mut stdout,
        &mut state.entries,
        &state.page_entries,
        &state.current_page,
        state.selected,
        state.scroll,
    )
    .await?;

    while let Some(Ok(msg)) = rx.next().await {
        // run action handlers which will decide what to redraw
        state.redraw = RedrawType::None;

        state.redraw = general::action_handler(&msg, &mut state).await;

        match msg {
            Letter::PeakVolumeUpdate(_,_) => {},
            _ => {
                log::error!("{:?}", state.redraw);
            }
        };

        if state.redraw == RedrawType::Exit {
            break;
        }

        let proposed_redraw = match state.ui_mode {
            UIMode::Normal => {
                let mut rdrw = normal::action_handler(&msg, &mut state).await;
                if state.current_page != PageType::Cards {
                    rdrw.take_bigger(play_entries::action_handler(&msg, &mut state).await);
                }
                rdrw
            },
            UIMode::ContextMenu => context_menu::action_handler(&msg, &mut state).await,
        };
        match msg {
            Letter::PeakVolumeUpdate(_,_) => {},
            _ => {
                log::error!("{:?}", state.redraw);
            }
        };

        state.redraw.take_bigger(proposed_redraw);
        match msg {
            Letter::PeakVolumeUpdate(_,_) => {},
            _ => {
                log::error!("{:?}", state.redraw);
            }
        };

        update_page_entries(&mut state).await?;

        redraw(&mut stdout, &mut state).await?;
    }
    Ok(())
}

async fn update_page_entries(state: &mut UIState) -> Result<(), RSError> {
    let last_sel = if state.selected < state.page_entries.len() {
        Some(state.page_entries.get(state.selected).unwrap().clone())
    } else {
        None
    };

    let (p, _) = parent_child_types(state.current_page);
    if !state.page_entries.set(
        state
            .current_page
            .generate_page(&state.entries)
            .map(|x| *x.0)
            .collect::<Vec<EntryIdentifier>>(),
        p,
    ) {
        state.redraw.take_bigger(RedrawType::Entries);

        DISPATCH
            .event(Letter::CreateMonitors(
                    if state.current_page != PageType::Cards {
                        monitor_list(state)
                    } else {
                        HashMap::new()
                    })).await;
    }

    for (i, x) in state.page_entries.iter_entries().enumerate() {
        if Some(*x) == last_sel {
            state.selected = i;
            break;
        }
    }

    if state
        .page_entries
        .adjust_scroll(&mut state.scroll, &mut state.selected)?
        && state.redraw != RedrawType::Full
        && state.redraw != RedrawType::ContextMenu
    {
        state.redraw = RedrawType::Entries;
    }

    Ok(())
}

fn monitor_list(state: &mut UIState) -> HashMap<EntryIdentifier, Option<u32>> {
    let mut monitors = HashMap::new();
    state.page_entries.iter_entries().for_each(|ident| {
        let mut monitor_src = None;
        if let Some(entry) = state.entries.get(ident) {
            match ident.entry_type {
                EntryType::SinkInput => {
                    if let Some(sink) = entry.play_entry.as_ref().unwrap().sink {
                        if let Some(s) = state
                            .entries
                            .get(&EntryIdentifier::new(EntryType::Sink, sink))
                        {
                            monitor_src = s.play_entry.as_ref().unwrap().monitor_source;
                        }
                    }
                }
                _ => {
                    monitor_src = entry.play_entry.as_ref().unwrap().monitor_source;
                }
            };

            monitors.insert(
                EntryIdentifier::new(entry.entry_type, entry.index),
                monitor_src,
            );
        }
    });

    monitors
}
