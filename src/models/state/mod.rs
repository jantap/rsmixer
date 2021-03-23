mod page_entries;

use super::{ContextMenu, PageEntries, PageType, Redraw, UIMode, PulseAudioAction, ContextMenuEffect};

use crate::{
    actor_system::Ctx,
    entry::{Entry, Entries, EntryIdentifier, EntryKind},
    ui::{
        widgets::{HelpWidget, WarningTextWidget},
        Scrollable,
        UI,
    },
};

use std::collections::HashMap;

use pulse::volume;

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
    pub ctx: Option<Ctx>,
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
            ctx: None,
        }
    }
}

impl RSState {
    pub fn new(ctx: Ctx) -> Self {
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
            ctx: Some(ctx),
        }
    }
    pub fn reset(&mut self) {
        self.ctx().send_to("pulseaudio", PulseAudioAction::CreateMonitors(HashMap::new()));
        *self = Self::new(self.ctx.take().unwrap());
        self.redraw.resize = true;
    }
    pub fn change_ui_mode(&mut self, mode: UIMode) {
        self.ui_mode = mode;
        self.redraw.resize = true;
    }
    pub fn remove_entry(&mut self, ident: &EntryIdentifier) {
        self.entries.remove(ident);

        if self.page_entries.ident_position(*ident).is_some() {
            page_entries::update(self);
        }
        
        if self.ui_mode == UIMode::ContextMenu {
            self.change_ui_mode(UIMode::Normal);
        }
    }

    pub fn update_entry(&mut self, ident: &EntryIdentifier, mut entry: Entry) {
        if entry.needs_redraw(&self.entries) {
            if let Some(i) = self
                .page_entries
                .iter_entries()
                .position(|id| *id == entry.entry_ident)
            {
                self.redraw.affected_entries.insert(i);
            }
        }

        entry.inherit_area(&self.entries);

        self.entries.insert(*ident, entry);

        page_entries::update(self);
    }

    pub fn update_peak_volume(&mut self, ident: &EntryIdentifier, peak: &f32) {
        if let Some(play) = self.entries.get_play_entry_mut(ident) {
            if (play.peak - peak).abs() < f32::EPSILON {
                return;
            }
            play.peak = *peak;

            if let Some(i) = self.page_entries.iter_entries().position(|&i| *ident == i) {
                self.redraw.peak_volume = Some(i);
            }
        }
    }

    pub fn move_down(&mut self, how_much: usize) {
        match self.ui_mode {
            UIMode::Normal => {
                self.selected_entry_needs_redraw();
                self.page_entries.down(how_much);
                self.selected_entry_needs_redraw();

                page_entries::update(self);
            }
            UIMode::ContextMenu => {
                self.context_menu.down(how_much);
                self.context_menu.horizontal_scroll = 0;

                self.redraw.context_menu = true;
            }
            UIMode::Help => {
                self.help.down(how_much);

                self.redraw.context_menu = true;
            }
            UIMode::MoveEntry(_, _) => {
                if self.page_entries.entries.len() < 2 {
                    return;
                }
                let l = self.page_entries.len() - 1;
                let selected = self.page_entries.selected() - 1;

                let mut j = (selected + how_much as usize) % l;

                if j >= selected {
                    j += 1;
                }

                let entry_ident = self.page_entries.get_selected().unwrap();
                let new_parent = self.page_entries.get(j as usize).unwrap();
                self.change_ui_mode(UIMode::MoveEntry(entry_ident, new_parent));

                page_entries::update(self);
            }
            _ => {},
        }
    }

    pub fn move_up(&mut self, how_much: usize) {
        match self.ui_mode {
            UIMode::Normal => {
                self.selected_entry_needs_redraw();
                self.page_entries.up(how_much);
                self.selected_entry_needs_redraw();

                page_entries::update(self);
            }
            UIMode::ContextMenu => {
                self.context_menu.up(how_much);
                self.context_menu.horizontal_scroll = 0;

                self.redraw.context_menu = true;
            }
            UIMode::Help => {
                self.help.up(how_much);

                self.redraw.context_menu = true;
            }
            UIMode::MoveEntry(_, _) => {
                if self.page_entries.entries.len() < 2 {
                    return;
                }
                let l = (self.page_entries.len() - 1) as i32;
                let selected = (self.page_entries.selected() - 1) as i32;

                let mut j = selected - how_much as i32;

                if j < 0 {
                    j = j.abs() % l;
                    j = l - j;
                }

                if j >= selected {
                    j += 1;
                }

                let entry_ident = self.page_entries.get_selected().unwrap();
                let new_parent = self.page_entries.get(j as usize).unwrap();
                self.change_ui_mode(UIMode::MoveEntry(entry_ident, new_parent));

                page_entries::update(self);
            }
            _ => {},
        }
    }

    pub fn move_left(&mut self) {
        if self.context_menu.horizontal_scroll > 0 {
            self.context_menu.horizontal_scroll -= 1;

            self.redraw.context_menu = true;
        }
    }

    pub fn move_right(&mut self) {
        if self.context_menu.horizontal_scroll < self.context_menu.max_horizontal_scroll() {
            self.context_menu.horizontal_scroll += 1;

            self.redraw.context_menu = true;
        }
    }

    pub fn set_selected(&mut self, index: usize) {
        match self.ui_mode {
            UIMode::Normal => {
                self.selected_entry_needs_redraw();
                self.page_entries.set_selected(index);
                self.selected_entry_needs_redraw();

                page_entries::update(self);
            }
            UIMode::ContextMenu => {
                self.context_menu.set_selected(index);

                self.redraw.context_menu = true;
            }
            _ => {}
        }
    }

    pub fn request_mute(&mut self, ident: &Option<EntryIdentifier>) {
        let ident = match *ident {
            Some(i) => i,
            None => match self.page_entries.get_selected() {
                Some(sel) => sel,
                None => { return; }
            }
        };

        let mute = match self.entries.get_play_entry(&ident) {
            Some(p) => p.mute,
            None => {
                return;
            }
        };
        self.ctx().send_to("pulseaudio", PulseAudioAction::MuteEntry(ident, !mute));
    }

    pub fn request_change_volume(&mut self, how_much: i16, ident: &Option<EntryIdentifier>) {
        let ident = match *ident {
            Some(i) => i,
            None => match self.page_entries.get_selected() {
                Some(sel) => sel,
                None => { return; }
            }
        };

        if let Some(play) = self.entries.get_play_entry_mut(&ident) {
            let mut vols = play.volume;
            let avg = vols.avg().0;

            let base_delta =
                (volume::Volume::NORMAL.0 as f32 - volume::Volume::MUTED.0 as f32) / 100.0;

            let current_percent =
                ((avg - volume::Volume::MUTED.0) as f32 / base_delta).round() as u32;
            let target_percent = current_percent as i16 + how_much;

            let target = if target_percent < 0 {
                volume::Volume::MUTED.0
            } else if target_percent == 100 {
                volume::Volume::NORMAL.0
            } else if target_percent >= 150 {
                (volume::Volume::NORMAL.0 as f32 * 1.5) as u32
            } else if target_percent < 100 {
                volume::Volume::MUTED.0 + target_percent as u32 * base_delta as u32
            } else {
                volume::Volume::NORMAL.0 + (target_percent - 100) as u32 * base_delta as u32
            };

            for v in vols.get_mut() {
                v.0 = target;
            }
            self.ctx().send_to("pulseaudio", PulseAudioAction::SetVolume(ident, vols));
        }
    }

    pub fn open_context_menu(&mut self, ident: &Option<EntryIdentifier>) {
        if let Some(ident) = ident {
            if let Some(index) = self.page_entries.iter_entries().position(|i| *i == *ident) {
                self.page_entries.set_selected(index);

                page_entries::update(self);
            }
        }

        if self.page_entries.selected() < self.page_entries.len() {
            if let Some(entry) = self.entries.get(
                &self
                    .page_entries
                    .get(self.page_entries.selected())
                    .unwrap(),
            ) {
                self.ui_mode = UIMode::ContextMenu;
                self.context_menu = ContextMenu::new(entry);

                if let EntryKind::CardEntry(card) = &entry.entry_kind {
                    self
                        .context_menu
                        .set_selected(card.selected_profile.unwrap_or(0));
                }

                self.redraw.resize = true;
            }
        }
    }

    pub fn confirm_context_menu(&mut self) {
        let selected = match self.page_entries.get_selected() {
            Some(ident) => ident,
            None => {
                return;
            }
        };

        let answer = self.context_menu.resolve(selected, &self.ctx());

        match answer {
            ContextMenuEffect::None => {
                self.change_ui_mode(UIMode::Normal);
            }
            ContextMenuEffect::MoveEntry => {
                let (parent_type, _) = self.current_page.parent_child_types();
                let entry_ident = selected;

                if let Some(parent_id) =
                    self.entries.get_play_entry(&entry_ident).unwrap().parent
                {
                    let entry_parent = EntryIdentifier::new(parent_type, parent_id);
                    let parent_ident = match self.entries.find(|(&i, _)| i == entry_parent) {
                        Some((i, _)) => *i,
                        None => EntryIdentifier::new(parent_type, 0),
                    };

                    self.change_ui_mode(UIMode::MoveEntry(entry_ident, parent_ident));

                    page_entries::update(self);
                } else {
                    self.change_ui_mode(UIMode::Normal);
                }
            }
        };
    }

    pub fn hide_entry(&mut self, ident: &Option<EntryIdentifier>) {
        let ident = match *ident {
            Some(i) => i,
            None => match self.page_entries.get_selected() {
                Some(i) => i,
                None => { return; }
            }
        };

        self.entries.hide(ident);

        page_entries::update(self);
    }

    pub fn change_page(&mut self, page: PageType) {
        self.current_page = page;
        self.change_ui_mode(UIMode::Normal);
        page_entries::update(self);
    }

    pub fn ctx(&self) -> &Ctx {
        self.ctx.as_ref().unwrap()
    }
    fn selected_entry_needs_redraw(&mut self) {
        self
            .redraw
            .affected_entries
            .insert(self.page_entries.selected());
    }
}
