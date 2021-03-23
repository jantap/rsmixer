use crate::{models::{UserAction, PageType}, repeat, BINDINGS};

use std::{collections::HashSet, mem::discriminant};

#[derive(Debug, Clone)]
pub struct HelpLine {
    pub key_events: Vec<String>,
    pub category: String,
}

impl HelpLine {
    pub fn lines_needed(&self, mut w: u16) -> u16 {
        let mut i = 0;
        let mut j = 0;
        w -= self.category.len() as u16;
        self.key_events.iter().for_each(|ke| {
            if i + ke.len() as u16 + 1 > w {
                i = (ke.len() + 1) as u16;
                j += 1;
            } else {
                i += (ke.len() + 1) as u16;
            }
        });

        j + 1
    }

    pub fn as_lines(&self, mut w: u16) -> Vec<String> {
        let mut cur = "".to_string();
        let mut v = Vec::new();
        w -= self.category.len() as u16;
        self.key_events.iter().for_each(|ke| {
            if cur.len() + ke.len() + 1 > w as usize {
                v.push(cur.clone());
                cur = format!("{} ", ke);
            } else {
                cur = format!("{}{} ", cur, ke);
            }
        });

        v.push(cur);

        if !v.is_empty() {
            v[0] = format!(
                "{}{}{}",
                v[0],
                repeat!(" ", w as usize - v[0].len()),
                self.category
            );
        }
        v
    }
}

pub enum ActionMatcher {
    Any(UserAction),
    Concrete(UserAction),
}

impl ActionMatcher {
    fn is_matching(&self, l2: &UserAction) -> bool {
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
        if let UserAction::RequstChangeVolume(x, _) = v {
            volume_deltas.insert(x.abs());
        }
    }

    categories.push((
        "Navigation".to_string(),
        vec![
            ActionMatcher::Any(UserAction::MoveUp(0)),
            ActionMatcher::Any(UserAction::MoveDown(0)),
        ],
    ));

    for vd in volume_deltas {
        categories.push((
            format!("Change volume by {}", vd),
            vec![
                ActionMatcher::Concrete(UserAction::RequstChangeVolume(vd, None)),
                ActionMatcher::Concrete(UserAction::RequstChangeVolume(-vd, None)),
            ],
        ))
    }
    categories.push((
        "Mute/unmute".to_string(),
        vec![ActionMatcher::Concrete(UserAction::RequestMute(None))],
    ));
    categories.push((
        "Change page".to_string(),
        vec![ActionMatcher::Any(UserAction::ChangePage(PageType::Output))],
    ));
    categories.push((
        "Cycle pages".to_string(),
        vec![ActionMatcher::Any(UserAction::CyclePages(0))],
    ));
    categories.push((
        "Context menu".to_string(),
        vec![ActionMatcher::Any(UserAction::OpenContextMenu(None))],
    ));
    categories.push((
        "Quit".to_string(),
        vec![ActionMatcher::Any(UserAction::RequestQuit)],
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
                    hl.key_events.push(k.to_string());
                }
            }
        }
        help_lines.push(hl);
    }

    help_lines
}
