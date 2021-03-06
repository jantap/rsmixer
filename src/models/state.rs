use super::{ContextMenu, PageEntries, PageType, Redraw, UIMode};

use crate::{
    entry::Entries,
    ui::{
        widgets::{HelpWidget, WarningTextWidget},
        UI,
    },
};

pub struct RSState {
    pub current_page: PageType,
    pub entries: Entries,
    pub page_entries: PageEntries,
    pub context_menu: ContextMenu,
    pub ui_mode: UIMode,
    pub redraw: Redraw,
    pub help: HelpWidget,
    pub warning_text: WarningTextWidget,
    pub ui: UI,
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
            warning_text: WarningTextWidget {
                text: "".to_string(),
            },
            ui: UI::default(),
        }
    }
}

impl RSState {
    pub fn change_ui_mode(&mut self, mode: UIMode) {
        self.ui_mode = mode;
        self.redraw.resize = true;
    }
}
