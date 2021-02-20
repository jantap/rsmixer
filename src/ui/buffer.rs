use crate::models::Style;

use super::Rect;

use std::{collections::HashMap, io::Write};

use crossterm::{
    cursor, queue,
    style::{self, ContentStyle},
};

use itertools::Itertools;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Pixel {
    pub text: Option<char>,
    pub style: Style,
}

impl Default for Pixel {
    fn default() -> Self {
        Self {
            text: None,
            style: Style::default(),
        }
    }
}

pub struct Buffer {
    pub width: u16,
    pub height: u16,
    pixels: Vec<Pixel>,
    changes: HashMap<usize, Pixel>,
    pub styles: HashMap<Style, ContentStyle>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            pixels: Vec::new(),
            changes: HashMap::new(),
            styles: HashMap::new(),
        }
    }
}

impl Buffer {
    pub fn set_styles(&mut self, styles: HashMap<Style, ContentStyle>) {
        self.styles = styles;
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.pixels = (0..width * height).map(|_| Pixel::default()).collect();
    }

    pub fn draw_changes<W: Write>(&mut self, stdout: &mut W) -> Result<(), crossterm::ErrorKind> {
        let mut changes = self.changes.clone();
        changes.retain(|k, v| {
            if let Some(pixel) = self.pixels.get(*k) {
                if *pixel == *v {
                    false
                } else {
                    true
                }
            } else {
                false
            }
        });

        self.changes = changes;

        let changes = self.changes.keys().sorted().collect_vec();

        let mut last = None;
        let mut i = 0;
        while i < changes.len() {
            if self.pixels.get(*changes[i]).is_none() {
                i += 1;
                continue;
            }

            let start_i = i;

            // decide whether moving the cursor is needed
            if last.is_none() || last.unwrap() + 1 != *changes[i] {
                let (x, y) = self.coord_to_xy(*changes[i]);
                queue!(stdout, cursor::MoveTo(x, y))?;
            }

            // take as many changes with the same style as possible

            let style = self.changes.get(changes[i]).unwrap().style;
            while i + 1 < changes.len() {
                if changes[i] + 1 != *changes[i + 1]
                    || self.pixels.get(*changes[i + 1]).is_none()
                    || self.changes.get(changes[i + 1]).unwrap().style != style
                {
                    break;
                }

                i += 1;
            }

            for k in &changes[start_i..i + 1] {
                self.pixels[**k] = *self.changes.get(*k).unwrap();
            }

            let range = *changes[start_i]..(*changes[i] + 1);
            let text = self.pixels[range]
                .iter()
                .map(|pixel| pixel.text.unwrap_or(' '))
                .collect::<String>();

            let style = match self.styles.get(&style) {
                Some(s) => *s,
                None => ContentStyle::default(),
            };

            queue!(stdout, style::PrintStyledContent(style.apply(text)))?;

            last = Some(*changes[i]);
            i += 1;
        }

        stdout.flush()?;

        Ok(())
    }

    fn coord_to_xy(&self, coord: usize) -> (u16, u16) {
        let y = (coord as f32 / self.width as f32).floor() as usize;
        let x = coord - (y * self.width as usize);
        (x as u16, y as u16)
    }

    fn xy_to_coord(&self, x: u16, y: u16) -> usize {
        (y * self.width + x) as usize
    }

    pub fn rect(&mut self, rect: Rect, text: char, style: Style) {
        let text: String = (0..rect.width).map(|_| text).collect();
        for y in 0..rect.height {
            self.string(rect.x, rect.y + y, text.clone(), style);
        }
    }

    pub fn string(&mut self, x: u16, y: u16, text: String, style: Style) {
        let coord = self.xy_to_coord(x, y);

        for (i, c) in text.chars().enumerate() {
            if i + coord >= self.pixels.len() {
                break;
            }

            self.pixel(
                coord + i,
                Pixel {
                    text: Some(c),
                    style,
                },
            );
        }
    }

    pub fn pixels(&mut self, x: u16, y: u16, pixels: Vec<Pixel>) {
        let coord = self.xy_to_coord(x, y);

        for (i, p) in pixels.iter().enumerate() {
            if i + coord >= self.pixels.len() {
                break;
            }

            self.pixel(coord + i, *p);
        }
    }

    pub fn pixel(&mut self, coord: usize, pixel: Pixel) {
        if self.pixels[coord] != pixel {
            self.changes.insert(coord, pixel);
        } else {
            self.changes.remove(&coord);
        }
    }
}
