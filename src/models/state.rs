use super::{ContextMenu, PageEntries, PageType, Redraw, UIMode};

use crate::{entry::Entries, ui::widgets::HelpWidget};

pub struct RSState {
    pub current_page: PageType,
    pub entries: Entries,
    pub page_entries: PageEntries,
    pub context_menu: ContextMenu,
    pub ui_mode: UIMode,
    pub redraw: Redraw,
    pub help: HelpWidget,
}

impl Default for RSState {
    fn default() -> Self {
        Self {
            current_page: PageType::Output,
            entries: Entries::default(),
            page_entries: PageEntries::new(),
            context_menu: ContextMenu::default(),
            ui_mode: UIMode::Normal,
            redraw: Redraw::default(),
            help: HelpWidget::default(),
        }
    }
}
