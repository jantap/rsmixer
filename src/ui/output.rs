use super::action_handlers::*;
pub use super::widgets::VolumeWidget;
use crate::entry::{Entries, EntryIdentifier, EntryType};
use crate::ui::draw::{draw_page, redraw};
use crate::ui::models::{ContextMenuOption, PageEntries};
use crate::{RSError, DISPATCH};
use std::collections::HashMap;

use super::util::Y_PADDING;
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
    execute!(stdout, Hide)?;

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
        log::error!("cur letter {:?}", msg.clone());
        // run action handlers which will decide what to redraw
        state.redraw = RedrawType::None;

        state.redraw = general::action_handler(&msg, &mut state).await;

        if state.redraw == RedrawType::Exit {
            break;
        }

        let r = entries_updates::action_handler(&msg, &mut state).await;
        log::error!("{:?}", r);
        state.redraw.take_bigger(r);


        let proposed_redraw = match state.ui_mode {
            UIMode::Normal => {
                let mut rdrw = normal::action_handler(&msg, &mut state).await;
                if state.current_page != PageType::Cards {
                    rdrw.take_bigger(play_entries::action_handler(&msg, &mut state).await);
                }
                rdrw
            }
            UIMode::ContextMenu => context_menu::action_handler(&msg, &mut state).await,
        };

        state.redraw.take_bigger(proposed_redraw);

        let r = scroll::scroll_handler(&msg, &mut state).await?;
        state.redraw.take_bigger(r);

        redraw(&mut stdout, &mut state).await?;
    }
    Ok(())
}

