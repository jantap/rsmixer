use crate::{config::keys, models::PageType, Letter, RSError, BINDINGS};

use std::{collections::HashSet, mem::discriminant};

#[derive(Debug)]
pub struct HelpLine {
    key_events: Vec<String>,
    category: String,
}

pub enum LetterMatcher {
    Any(Letter),
    Concrete(Letter),
}

impl LetterMatcher {
    fn is_matching(&self, l2: &Letter) -> bool {
        match self {
            Self::Any(l1) => discriminant(l1) == discriminant(l2),
            Self::Concrete(l1) => l1 == l2,
        }
    }
}

pub fn generate() -> Vec<HelpLine> {
    let mut categories = Vec::new();

    let mut volume_deltas = HashSet::new();

    for (_, v) in (*BINDINGS).get().iter() {
        if let Letter::RequstChangeVolume(x) = v {
            volume_deltas.insert(x.abs());
        }
    }

    categories.push((
        "Navigation".to_string(),
        vec![
            LetterMatcher::Any(Letter::MoveUp(0)),
            LetterMatcher::Any(Letter::MoveDown(0)),
        ],
    ));

    for vd in volume_deltas {
        categories.push((
            format!("Change volume by {}", vd),
            vec![
                LetterMatcher::Concrete(Letter::RequstChangeVolume(vd)),
                LetterMatcher::Concrete(Letter::RequstChangeVolume(-vd)),
            ],
        ))
    }
    categories.push((
        "Mute/unmute".to_string(),
        vec![LetterMatcher::Concrete(Letter::RequestMute)],
    ));
    categories.push((
        "Change page".to_string(),
        vec![LetterMatcher::Any(Letter::ChangePage(PageType::Output))],
    ));
    categories.push((
        "Cycle pages".to_string(),
        vec![LetterMatcher::Any(Letter::CyclePages(0))],
    ));
    categories.push((
        "Context menu".to_string(),
        vec![LetterMatcher::Any(Letter::OpenContextMenu)],
    ));
    categories.push((
        "Quit".to_string(),
        vec![LetterMatcher::Any(Letter::ExitSignal)],
    ));

    let mut help_lines = Vec::new();

    for category in categories {
        let mut hl = HelpLine {
            key_events: Vec::new(),
            category: category.0,
        };
        for (k, v) in (*BINDINGS).get().iter() {
            for matcher in &category.1 {
                if matcher.is_matching(v) {
                    hl.key_events.push(keys::keyevent_to_string(k));
                }
            }
        }
        help_lines.push(hl);
    }

    help_lines
}

pub fn help_lines_to_strings(hls: &[HelpLine], width: u16) -> Result<(u16, Vec<String>), RSError> {
    let mut lines = Vec::new();

    let longest_name = hls.iter().map(|hl| hl.category.len()).max().unwrap();
    let longest_first_two = hls
        .iter()
        .map(|hl| {
            let mut c = 0;
            if !hl.key_events.is_empty() {
                c += hl.key_events[0].len();
            }
            if hl.key_events.len() > 1 {
                c += hl.key_events[1].len();
            }
            c
        })
        .max()
        .unwrap();
    let longest_key_ev = hls
        .iter()
        .map(|hl| hl.key_events.iter().map(|ev| ev.len()).max().unwrap())
        .max()
        .unwrap();

    let big_width = longest_name + longest_first_two + 5;
    let small_width = longest_name + longest_key_ev + 3;

    if width < small_width as u16 {
        return Err(RSError::TerminalTooSmall);
    }
    let width = if width > big_width as u16 {
        big_width
    } else {
        small_width
    };

    for hl in hls {
        let available_width = width - longest_name - 3;
        let mut current_lines = Vec::new();
        current_lines.push("".to_string());

        for ev in &hl.key_events {
            let cond = available_width >= current_lines.last().unwrap().len() + ev.len();
            if cond {
                let cur = current_lines.last_mut().unwrap();
                *cur = format!("{}{}  ", cur, ev);
            } else {
                current_lines.push(format!("{}  ", ev.clone()));
            }
        }

        let first = current_lines.first_mut().unwrap();
        let whitespace_needed = width - longest_name - (*first).len();
        let whitespace = (0..whitespace_needed).map(|_| " ").collect::<String>();
        *first = format!("{}{}{}", first.clone(), whitespace, hl.category);

        lines.append(&mut current_lines);
    }

    Ok((width as u16, lines))
}
