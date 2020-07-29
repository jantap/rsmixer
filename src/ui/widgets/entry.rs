use crate::{
    draw_at,
    entry::{Entry, EntrySpaceLvl},
    ui::{
        util::{get_style, Rect},
        widgets::{VolumeWidgetBorder, Widget},
    },
    RSError,
};

use pulse::volume;

use std::io::Write;

use crossterm::{cursor::MoveTo, execute};

impl<W: Write> Widget<W> for Entry {
    fn render(&mut self, area: Rect, buf: &mut W) -> Result<(), RSError> {
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

        let mut area_a = Entry::calc_area(self.position, area.clone());
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

        let main_vol = (self.volume.avg().0 as f32) / (volume::VOLUME_NORM.0 as f32 * 1.5);
        self.volume_bar = self
            .volume_bar
            .volume(main_vol)
            .mute(self.mute)
            .border(VolumeWidgetBorder::Upper);
        self.peak_volume_bar = self.peak_volume_bar.mute(self.mute).volume(self.peak);

        let short_name = self
            .name
            .chars()
            .take(area1.width as usize - 2)
            .collect::<String>();

        execute!(buf, MoveTo(area1.x, area1.y))?;
        write!(buf, "{}", name_style.clone().apply(short_name))?;

        let vol_perc = format!("  {}", (main_vol * 150.0) as u32);
        let vol_perc = String::from(&vol_perc[vol_perc.len() - 3..vol_perc.len()]);
        let vol_db = self.volume.avg().print_db();
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
            draw_at!(buf, c, area2.x - 1, area2.y + 1, style.clone());

            self.volume_bar.render(area2, buf)?;
            area2.y += 1;
            self.volume_bar
                .border(VolumeWidgetBorder::Lower)
                .render(area2, buf)?;
        }

        area_a.y += 2;
        area_a.height = 1;
        area_a.width -= 1;
        self.peak_volume_bar.render(area_a, buf)?;

        buf.flush()?;

        Ok(())
    }
}
