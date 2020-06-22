use crate::draw_at;
use super::{volume::VolumeWidgetBorder, VolumeWidget};

use crate::{
    ui::{
        entries::Entry,
        util::{get_style, Rect},
        visibility::EntrySpaceLvl,
        Widget,
    },
    Result,
};

use std::io::Write;

use crossterm::{cursor::MoveTo, execute};

use pulse::volume;

pub struct EntryWidget<'a> {
    pub entry: &'a Entry,
    pub volume_bar: VolumeWidget,
    pub peak_volume_bar: VolumeWidget,
    pub bold: bool,
    pub mute: bool,
    pub position: EntrySpaceLvl,
}

impl<'a> EntryWidget<'a> {
    pub fn from(entry: &'a Entry) -> Self {
        Self {
            entry,
            volume_bar: VolumeWidget::default(),
            peak_volume_bar: VolumeWidget::default(),
            mute: entry.mute,
            bold: false,
            position: EntrySpaceLvl::Parent,
        }
    }
    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }
    pub fn position(mut self, position: EntrySpaceLvl) -> Self {
        self.position = position;
        self
    }
    pub fn calc_area(position: EntrySpaceLvl, mut area: Rect) -> Rect {
        let amount = match position {
            EntrySpaceLvl::Parent => 2,
            EntrySpaceLvl::ParentNoChildren => 2,
            _ => 5,
        };

        area.x += amount;
        area.width -= amount;

        area
    }
}

impl<'a, W: Write> Widget<W> for EntryWidget<'a> {
    fn render(mut self, area: Rect, buf: &mut W) -> Result<()> {
        let style = if self.bold { "bold" } else { "normal" };
        let style = get_style(style);

        let area1: Rect;
        let mut area2: Rect;

        let mut area_a = EntryWidget::calc_area(self.position, area.clone());

        let seventy_percent = (area_a.width as f32 * 0.7).floor() as u16;
        if area_a.width - seventy_percent > 30 {
            area1 = Rect::new(area_a.x, area_a.y, area_a.width - seventy_percent, 1);
            area2 = Rect::new(
                area_a.x + area_a.width - seventy_percent,
                area_a.y,
                seventy_percent,
                1,
            );
        } else {
            area1 = Rect::new(area_a.x, area_a.y, 30, 1);
            area2 = Rect::new(area_a.x + 30, area_a.y, area_a.width - 30, 1);
        }

        area2.width -= 2;
        area2.x += 1;

        let main_vol = (self.entry.volume.avg().0 as f32) / (volume::VOLUME_NORM.0 as f32 * 1.5);
        self.volume_bar = self
            .volume_bar
            .volume(main_vol)
            .mute(self.mute)
            .border(VolumeWidgetBorder::Upper);
        self.peak_volume_bar = self.peak_volume_bar.mute(self.mute).volume(self.entry.peak);

        let vol_perc = format!("  {}", (main_vol * 150.0) as u32);
        let vol_perc = String::from(&vol_perc[vol_perc.len() - 3..vol_perc.len()]);

        let short_name = self
            .entry
            .name
            .chars()
            .take(area1.width as usize - 2)
            .collect::<String>();

        execute!(buf, MoveTo(area1.x, area1.y))?;
        write!(buf, "{}", style.clone().apply(short_name))?;

        execute!(buf, MoveTo(area1.x + area1.width - 5, area1.y + 1))?;
        write!(buf, "{}", style.clone().apply(vol_perc))?;


        let c = if self.bold { "-" } else { " " };

        draw_at!(buf, c, area2.x+area2.width, area2.y, style.clone());
        draw_at!(buf, c, area2.x-1, area2.y, style.clone());
        draw_at!(buf, c, area2.x+area2.width, area2.y + 1, style.clone());
        draw_at!(buf, c, area2.x-1, area2.y + 1, style.clone());

        self.volume_bar.render(area2, buf)?;
        area2.y += 1;
        self.volume_bar
            .border(VolumeWidgetBorder::Lower)
            .render(area2, buf)?;

        area_a.y += 2;
        area_a.height = 1;
        self.peak_volume_bar.render(area_a, buf)?;

        buf.flush()?;

        Ok(())
    }
}
