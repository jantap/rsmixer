
use crossterm::style::Color;

pub fn str_to_color(s: &str) -> Option<Color> {
    if s.chars().take(1).collect::<String>() == "#" && s.len() == 7 {
        Some(Color::Rgb {
            r: u8::from_str_radix(&s[1..3], 16).expect("error in config"),
            g: u8::from_str_radix(&s[3..5], 16).expect("error in config"),
            b: u8::from_str_radix(&s[5..7], 16).expect("error in config"),
        })
    } else {
        match &s[..].parse::<Color>() {
            Ok(c) => Some(*c),
            Err(_) => None,
        }
    }
}
