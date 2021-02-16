use crate::{
    entry::{CardEntry, Entry, EntryKind, EntrySpaceLvl, HiddenStatus, PlayEntry},
    ui::{
        widgets::{VolumeWidgetBorder, Widget},
        Screen, Style,
    },
    RSError,
};

use screen_buffer_ui::Rect;

use std::cmp::min;

use pulse::volume;

impl Widget for Entry {
    fn resize(&mut self, area: Rect) -> Result<(), RSError> {
        if area.width < 7 || area.height < 1 {
            return Err(RSError::TerminalTooSmall);
        }

        match &mut self.entry_kind {
            EntryKind::PlayEntry(play) => {
                play.position = self.position;
                play.resize(area)
            }
            EntryKind::CardEntry(card) => card.resize(area),
        }
    }
    fn render(&mut self, screen: &mut Screen) -> Result<(), RSError> {
        match &mut self.entry_kind {
            EntryKind::PlayEntry(play) => {
                play.is_selected = self.is_selected;
                play.position = self.position;

                play.render(screen)
            }
            EntryKind::CardEntry(card) => {
                card.is_selected = self.is_selected;

                card.render(screen)
            }
        }
    }
}

impl CardEntry {
    pub fn set_is_selected(mut self, is_selected: bool) -> Self {
        self.is_selected = is_selected;
        self
    }
}

impl PlayEntry {
    pub fn set_is_selected(mut self, is_selected: bool) -> Self {
        self.is_selected = is_selected;
        self
    }
    pub fn position(mut self, position: EntrySpaceLvl) -> Self {
        self.position = position;
        self
    }

    fn is_volume_visible(&self) -> bool {
        let (_, w) = self.text_volume_widths();
        w > 0
    }

    fn text_volume_widths(&self) -> (u16, u16) {
        let w = self.area.width - self.offset();
        let text_width = if w > 100 {
            (w as f32 * 0.3).floor() as u16
        } else if w > 35 {
            35
        } else {
            w
        };
        (text_width, w - text_width)
    }

    fn play_entry_text_area(&self) -> Rect {
        let (w, _) = self.text_volume_widths();
        Rect::new(self.area.x + self.offset(), self.area.y, w, 2)
    }

    fn offset(&self) -> u16 {
        match self.position {
            EntrySpaceLvl::Parent | EntrySpaceLvl::ParentNoChildren => 2,
            EntrySpaceLvl::MidChild | EntrySpaceLvl::LastChild => 5,
            _ => 0,
        }
    }
}

impl Widget for CardEntry {
    fn resize(&mut self, area: Rect) -> Result<(), RSError> {
        self.area = area;
        Ok(())
    }

    fn render(&mut self, screen: &mut Screen) -> Result<(), RSError> {
        screen.rect(self.area, ' ', Style::Normal);

        let style = if self.is_selected {
            Style::Bold
        } else {
            Style::Normal
        };
        let name_style = if self.is_selected {
            Style::Inverted
        } else {
            Style::Normal
        };

        let name_len = min(self.name.len(), (self.area.width / 2).into());

        screen.string(
            self.area.x,
            self.area.y,
            (&self.name[0..name_len]).to_string(),
            name_style,
        );

        if let Some(index) = self.selected_profile {
            let profile_len = min(
                self.profiles[index].description.len(),
                (self.area.width / 2).into(),
            );

            screen.string(
                self.area.x + self.area.width - profile_len as u16,
                self.area.y,
                (&self.profiles[index].description[0..profile_len]).to_string(),
                style,
            );
        }

        Ok(())
    }
}
impl Widget for PlayEntry {
    fn resize(&mut self, area: Rect) -> Result<(), RSError> {
        self.area = area;
        let (text_width, w) = self.text_volume_widths();

        if w > 0 {
            let volume_area = Rect::new(
                self.area.x + text_width + self.offset() + 1,
                self.area.y,
                w - 2,
                1,
            );
            self.volume_bar = self.volume_bar.set_area(volume_area);
        }

        let y = match self.position {
            EntrySpaceLvl::ParentNoChildren | EntrySpaceLvl::LastChild => {
                self.area.y + self.area.height - 2
            }
            _ => self.area.y + self.area.height - 1,
        };

        self.peak_volume_bar = self.peak_volume_bar.set_area(Rect::new(
            self.area.x + self.offset(),
            y,
            self.area.width - self.offset() - 1,
            1,
        ));

        Ok(())
    }

    fn render(&mut self, screen: &mut Screen) -> Result<(), RSError> {
        if self.area.width < 5 || self.area.height < 2 {
            return Err(RSError::TerminalTooSmall);
        }

        screen.rect(self.area, ' ', Style::Normal);

        let style = if self.is_selected {
            Style::Bold
        } else {
            Style::Normal
        };
        let name_style = if self.is_selected {
            Style::Inverted
        } else {
            Style::Normal
        };

        let text_area = self.play_entry_text_area();
        let short_name = self
            .name
            .chars()
            .take(if text_area.width > 2 {
                text_area.width as usize - 2
            } else {
                0
            })
            .collect::<String>();

        screen.string(text_area.x, text_area.y, short_name, name_style);

        let avg = self.volume.avg().0;
        let base_delta = (volume::Volume::NORMAL.0 as f32 - volume::Volume::MUTED.0 as f32) / 100.0;
        let vol_percent = ((avg - volume::Volume::MUTED.0) as f32 / base_delta).round() as u32;

        if self.is_volume_visible() {
            let volume_area = self.volume_bar.area;
            self.volume_bar = self
                .volume_bar
                .volume(vol_percent as f32 / 150.0)
                .mute(self.mute)
                .border(VolumeWidgetBorder::Upper);

            self.volume_bar.render(screen)?;

            self.volume_bar.area = self.volume_bar.area.y(volume_area.y + 1);

            self.volume_bar = self.volume_bar.border(VolumeWidgetBorder::Lower);

            self.volume_bar.render(screen)?;

            self.volume_bar.area = self.volume_bar.area.y(volume_area.y);

            if self.is_selected {
                let c = "-".to_string();
                screen.string(volume_area.x - 1, volume_area.y, c.clone(), style);
                screen.string(volume_area.x - 1, volume_area.y + 1, c.clone(), style);
                screen.string(
                    volume_area.x + volume_area.width,
                    volume_area.y,
                    c.clone(),
                    style,
                );
                screen.string(
                    volume_area.x + volume_area.width,
                    volume_area.y + 1,
                    c,
                    style,
                );
            }
        }

        let vol_perc = format!("  {}", vol_percent);
        let vol_perc = String::from(&vol_perc[vol_perc.len() - 3..vol_perc.len()]);
        let vol_db = self.volume.avg().print_db();

        if vol_db.len() + vol_perc.len() <= text_area.width as usize + 3 {
            let vol_str = format!(
                "{}{}{}",
                vol_db,
                (0..text_area.width as usize - 3 - vol_perc.len() - vol_db.len())
                    .map(|_| " ")
                    .collect::<String>(),
                vol_perc
            );

            screen.string(text_area.x + 1, text_area.y + 1, vol_str, style);
        }

        self.peak_volume_bar.mute = self.mute;
        self.peak_volume_bar.render(screen)?;

        match self.position {
            EntrySpaceLvl::Parent => {
                screen.string(self.area.x, self.area.y, "▼".to_string(), style);
                screen.string(self.area.x, self.area.y + 1, "│".to_string(), style);
                screen.string(self.area.x, self.area.y + 2, "│".to_string(), style);
            }
            EntrySpaceLvl::ParentNoChildren => match self.hidden {
                HiddenStatus::HiddenKids => {
                    screen.string(self.area.x, self.area.y, "▲".to_string(), style);
                }
                HiddenStatus::NoKids => {
                    screen.string(self.area.x, self.area.y, "▶".to_string(), style);
                }
                _ => {}
            },
            EntrySpaceLvl::MidChild => {
                screen.string(self.area.x, self.area.y, "│".to_string(), style);
                screen.string(self.area.x, self.area.y + 1, "│".to_string(), style);
                screen.string(self.area.x, self.area.y + 2, "├───".to_string(), style);
            }
            EntrySpaceLvl::LastChild => {
                screen.string(self.area.x, self.area.y, "│".to_string(), style);
                screen.string(self.area.x, self.area.y + 1, "│".to_string(), style);
                screen.string(self.area.x, self.area.y + 2, "└───".to_string(), style);
            }
            _ => {}
        };

        Ok(())
    }
}
