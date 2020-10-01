use super::Widget;

use crate::{
    draw_at, repeat_string,
    ui::util::{get_style, Rect},
    RSError,
};

use std::{
    cmp::{max, min},
    io::Write,
};

use crossterm::{cursor::MoveTo, execute};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum VolumeWidgetBorder {
    Single,
    Upper,
    Lower,
    None,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct VolumeWidget {
    pub percent: f32,
    pub border: VolumeWidgetBorder,
    last_filled: u16,
    last_area: Rect,
    pub mute: bool,
}

impl VolumeWidget {
    pub fn default() -> Self {
        Self {
            percent: 0.0,
            border: VolumeWidgetBorder::Single,
            last_filled: 0,
            last_area: Rect::default(),
            mute: false,
        }
    }

    pub fn volume(mut self, percent: f32) -> Self {
        self.percent = percent;
        self
    }

    pub fn border(mut self, border: VolumeWidgetBorder) -> Self {
        self.border = border;
        self
    }

    pub fn mute(mut self, mute: bool) -> Self {
        self.mute = mute;
        self
    }

    fn get_segments(&self, width: u16) -> (u16, u16, u16) {
        let third = (0.34 * (width - 2) as f32).floor() as u16;
        let last = width - 2 - third * 2;

        (third, third * 2, last)
    }
}

impl<W: Write> Widget<W> for VolumeWidget {
    fn render(&mut self, area: Rect, buf: &mut W) -> Result<(), RSError> {
        self.border.render(area, buf)?;

        let filled = (self.percent * (area.width - 2) as f32).floor() as u16;

        let segments = self.get_segments(area.width);
        let third = segments.0;

        let styles = if self.mute { 
            (get_style("muted"), get_style("muted"), get_style("muted"))
        } else {
            (get_style("green"), get_style("orange"), get_style("red"))
        };

        if self.last_area != area {
            let green_filled = min(filled, third);
            let orange_filled = max(min(filled, third * 2), third) - third;
            let red_filled = max(min(filled, area.width - 2), third * 2) - third * 2;

            let s = format!(
                "{}{}{}",
                styles.0.apply(format!(
                    "{}{}",
                    repeat_string!("▮", green_filled),
                    repeat_string!("-", segments.0 - green_filled),
                )),
                styles.1.apply(format!(
                    "{}{}",
                    repeat_string!("▮", orange_filled),
                    repeat_string!("-", segments.1 - orange_filled),
                )),
                styles.2.apply(format!(
                    "{}{}",
                    repeat_string!("▮", red_filled),
                    repeat_string!("-", segments.2 - red_filled),
                ))
            );
            execute!(buf, MoveTo(area.x + 1, area.y))?;
            write!(buf, "{}", s)?;
        } else {
            let (range, ch) = if filled > self.last_filled {
                (self.last_filled..filled, "▮")
            } else {
                (filled..self.last_filled, "-")
            };
            let mut g = 0;
            let mut o = 0;
            let mut r = 0;

            for i in range {
                if i < third {
                    g += 1;
                } else if i > third * 2 {
                    r += 1;
                } else {
                    o += 1;
                }
            }

            let s = format!(
                "{}{}{}",
                styles.0.apply(repeat_string!(ch, g)),
                styles.1.apply(repeat_string!(ch, r)),
                styles.2.apply(repeat_string!(ch, o)),
            );
            execute!(
                buf,
                MoveTo(area.x + 1 + min(self.last_filled, filled), area.y)
            )?;
            write!(buf, "{}", s)?;
        }

        buf.flush()?;

        Ok(())
    }
}

impl<W: Write> Widget<W> for VolumeWidgetBorder {
    fn render(&mut self, area: Rect, buf: &mut W) -> Result<(), RSError> {
        if *self == VolumeWidgetBorder::None {
            return Ok(());
        }

        let ch1 = match self {
            VolumeWidgetBorder::Single => "[",
            VolumeWidgetBorder::Upper => "┌",
            VolumeWidgetBorder::Lower => "└",
            _ => "",
        };
        let ch2 = match self {
            VolumeWidgetBorder::Single => "]",
            VolumeWidgetBorder::Upper => "┐",
            VolumeWidgetBorder::Lower => "┘",
            _ => "",
        };
        draw_at!(buf, ch1, area.x, area.y, get_style("normal"));
        draw_at!(
            buf,
            ch2,
            area.x + area.width - 1,
            area.y,
            get_style("normal")
        );

        Ok(())
    }
}
