use crate::{
    draw_at,
    entry::{Entry, EntrySpaceLvl, EntryType},
    ui::{
        util::{get_style, Rect},
        widgets::{VolumeWidgetBorder, Widget},
    },
    RSError,
};

use std::{cmp::min, io::Write};

use pulse::volume;

use crossterm::{cursor::MoveTo, execute};

impl<W: Write> Widget<W> for Entry {
    fn render(&mut self, area: Rect, buf: &mut W) -> Result<(), RSError> {
        if self.entry_type == EntryType::Card {
            self.render_card_entry(area, buf)
        } else {
            self.render_play_entry(area, buf)
        }
    }
}

impl Entry {
    fn render_card_entry<W: Write>(&mut self, area: Rect, buf: &mut W) -> Result<(), RSError> {
        let card = self.card_entry.as_mut().unwrap();

        let style = if self.is_selected {
            "normal.bold"
        } else {
            "normal"
        };
        let style = get_style(style);
        let name_style = if self.is_selected {
            "inverted"
        } else {
            "normal"
        };
        let name_style = get_style(name_style);

        let left_name_len = min(self.name.len(), (area.width / 2).into());

        execute!(buf, MoveTo(area.x, area.y))?;
        write!(buf, "{}", name_style.apply(&self.name[0..left_name_len]))?;

        if let Some(index) = card.selected_profile {
            let right_name_len = min(
                card.profiles[index].description.len(),
                (area.width / 2).into(),
            );
            execute!(
                buf,
                MoveTo(area.x + area.width - right_name_len as u16, area.y)
            )?;
            write!(
                buf,
                "{}",
                style.apply(&card.profiles[index].description[0..right_name_len])
            )?;
        }

        Ok(())
    }
    fn render_play_entry<W: Write>(&mut self, area: Rect, buf: &mut W) -> Result<(), RSError> {
        let play = self.play_entry.as_mut().unwrap();

        let style = if self.is_selected {
            "normal.bold"
        } else {
            "normal"
        };
        let style = get_style(style);
        let name_style = if self.is_selected {
            "inverted"
        } else {
            "normal"
        };
        let name_style = get_style(name_style);

        let area1: Rect;
        let mut area2: Rect;

        let a = area;
        let mut area_a = Entry::calc_area(self.position, a);
        let small = area_a.width <= 35;

        let seventy_percent = (area_a.width as f32 * 0.7).floor() as u16;
        if area_a.width > 30 + seventy_percent {
            area1 = Rect::new(area_a.x, area_a.y, area_a.width - seventy_percent, 1);
            area2 = Rect::new(
                area_a.x + area_a.width - seventy_percent + 1,
                area_a.y,
                seventy_percent - 2,
                1,
            );
        } else if area_a.width > 35 {
            area1 = Rect::new(area_a.x, area_a.y, 35, 1);
            area2 = Rect::new(area_a.x + 36, area_a.y, area_a.width - 37, 1);
        } else {
            area1 = Rect::new(area_a.x, area_a.y, area_a.width, 1);
            area2 = Rect::new(0, 0, 0, 0);
        }

        let main_vol = (play.volume.avg().0 as f32) / (volume::VOLUME_NORM.0 as f32 * 1.5);
        play.volume_bar = play
            .volume_bar
            .volume(main_vol)
            .mute(play.mute)
            .border(VolumeWidgetBorder::Upper);
        play.peak_volume_bar = play.peak_volume_bar.mute(play.mute).volume(play.peak);

        let short_name = self
            .name
            .chars()
            .take(area1.width as usize - 2)
            .collect::<String>();

        execute!(buf, MoveTo(area1.x, area1.y))?;
        write!(buf, "{}", name_style.apply(short_name))?;

        let vol_perc = format!("  {}", (main_vol * 150.0) as u32);
        let vol_perc = String::from(&vol_perc[vol_perc.len() - 3..vol_perc.len()]);
        let vol_db = play.volume.avg().print_db();
        if vol_db.len() + vol_perc.len() <= area1.width as usize + 3 {
            let vol_str = format!(
                "{}{}{}",
                vol_db,
                (0..area1.width as usize - 3 - vol_perc.len() - vol_db.len())
                    .map(|_| " ")
                    .collect::<String>(),
                vol_perc
            );

            execute!(buf, MoveTo(area1.x + 1, area1.y + 1))?;
            write!(buf, "{}", style.clone().apply(vol_str))?;
        }

        let mut v = Vec::new();
        match self.position {
            EntrySpaceLvl::Parent => {
                v.push("▼");
                v.push("│");
                v.push("│");
            }
            EntrySpaceLvl::ParentNoChildren => {
                v.push("▲");
            }
            EntrySpaceLvl::MidChild => {
                v.push("│");
                v.push("│");
                v.push("├───");
            }
            EntrySpaceLvl::LastChild => {
                v.push("│");
                v.push("│");
                v.push("└───");
            }
            _ => {}
        };

        for (i, q) in v.iter().enumerate() {
            execute!(buf, MoveTo(area.x, area.y + i as u16))?;
            write!(buf, "{}", style.clone().apply(q))?;
        }

        if !small {
            let c = if self.is_selected { "-" } else { " " };

            draw_at!(buf, c, area2.x + area2.width, area2.y, style.clone());
            draw_at!(buf, c, area2.x - 1, area2.y, style.clone());
            draw_at!(buf, c, area2.x + area2.width, area2.y + 1, style.clone());
            draw_at!(buf, c, area2.x - 1, area2.y + 1, style);

            play.volume_bar.render(area2, buf)?;
            area2.y += 1;
            play.volume_bar
                .border(VolumeWidgetBorder::Lower)
                .render(area2, buf)?;
        }

        area_a.y += 2;
        area_a.height = 1;
        area_a.width -= 1;
        play.peak_volume_bar.render(area_a, buf)?;

        buf.flush()?;

        Ok(())
    }
}
