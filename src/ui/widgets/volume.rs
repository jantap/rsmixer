use crate::{
    draw_at, repeat_string,
    ui::{
        util::{get_style, Rect},
        Widget,
    },
    Result,
};

use std::{
    cmp::{max, min},
    io::Write,
};

use crossterm::{cursor::MoveTo, execute};

#[derive(Copy, Clone, PartialEq)]
pub enum VolumeWidgetBorder {
    Single,
    Upper,
    Lower,
    None,
}

#[derive(Copy, Clone)]
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
}

impl<W: Write> Widget<W> for VolumeWidget {
    fn render(self, area: Rect, buf: &mut W) -> Result<()> {
        // draw_rect!(buf, " ", area, get_style("normal"));
        if self.border != VolumeWidgetBorder::None {
            let ch1 = match self.border {
                VolumeWidgetBorder::Single => "[",
                VolumeWidgetBorder::Upper => "┌",
                VolumeWidgetBorder::Lower => "└",
                _ => "",
            };
            let ch2 = match self.border {
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
        }

        let filled = (self.percent * (area.width - 2) as f32).floor() as u16;
        let third = (0.34 * (area.width - 2) as f32).floor() as u16;
        let last = area.width - 2 - third * 2;
        let g_style = if self.mute {
            get_style("muted")
        } else {
            get_style("green")
        };
        let o_style = if self.mute {
            get_style("muted")
        } else {
            get_style("orange")
        };
        let r_style = if self.mute {
            get_style("muted")
        } else {
            get_style("red")
        };

        if self.last_area != area {
            let green_filled = min(filled, third);
            let orange_filled = max(min(filled, third * 2), third) - third;
            let red_filled = max(min(filled, area.width - 2), third * 2) - third * 2;

            let s = format!(
                "{}{}{}",
                g_style.apply(format!(
                    "{}{}",
                    repeat_string!("▮", green_filled),
                    repeat_string!("-", third - green_filled),
                )),
                o_style.apply(format!(
                    "{}{}",
                    repeat_string!("▮", orange_filled),
                    repeat_string!("-", third - orange_filled),
                )),
                r_style.apply(format!(
                    "{}{}",
                    repeat_string!("▮", red_filled),
                    repeat_string!("-", last - red_filled),
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
                g_style.apply(repeat_string!(ch, g)),
                o_style.apply(repeat_string!(ch, r)),
                r_style.apply(repeat_string!(ch, o)),
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
