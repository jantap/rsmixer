use super::{PageEntries, PageType, ContextMenuOption, RedrawType, UIMode};

use crate::{
    entry::Entries,
    ui::{
        page::UIPage,
        util::Rect,
    },
};

pub struct RSState {
    pub current_page: PageType,
    pub entries: Entries,
    pub page_entries: PageEntries,
    pub selected: usize,
    pub selected_context: usize,
    pub context_options: Vec<ContextMenuOption>,
    pub scroll: usize,
    pub redraw: RedrawType,
    pub ui_mode: UIMode,
    pub ui_page: UIPage,
}

impl Default for RSState {
    fn default() -> Self {
        Self {
            current_page: PageType::Output,
            entries: Entries::default(),
            page_entries: PageEntries::new(),
            selected: 0,
            selected_context: 0,
            context_options: Vec::new(),
            scroll: 0,
            redraw: RedrawType::None,
            ui_mode: UIMode::Normal,
            ui_page: UIPage {
                inner_area: Rect::new(2, 2, 0, 0),
            },
        }
    }
}
