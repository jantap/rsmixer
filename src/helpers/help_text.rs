use crate::{Letter, BINDINGS, ui::PageType, helpers::keys};

use std::{mem::discriminant, collections::HashSet};

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
            Self::Any(l1) => {
                discriminant(l1) == discriminant(l2)
            }
            Self::Concrete(l1) => {
                l1 == l2
            }
        }
    }
}

pub fn generate() -> Vec<HelpLine> {
    let mut categories = Vec::new();

    let mut volume_deltas = HashSet::new();
    
    for (k, v) in (*BINDINGS).get().iter() {
        if let Letter::RequstChangeVolume(x) = v {
            volume_deltas.insert(x.abs());
        }
    }

    categories.push(("Navigation".to_string(), vec![
                      LetterMatcher::Any(Letter::MoveUp(0)), 
                      LetterMatcher::Any(Letter::MoveDown(0))]));

    for vd in volume_deltas {
        categories.push((format!("Change volume by {}", vd), vec![
                          LetterMatcher::Concrete(Letter::RequstChangeVolume(vd)),
                          LetterMatcher::Concrete(Letter::RequstChangeVolume(-vd))]))
    }
    categories.push(("Mute".to_string(), vec![LetterMatcher::Concrete(Letter::RequestMute)]));
    categories.push(("Change page".to_string(), vec![LetterMatcher::Any(Letter::ChangePage(PageType::Output))]));
    categories.push(("Cycle pages".to_string(), vec![LetterMatcher::Any(Letter::CyclePages(0))]));
    categories.push(("Context menu".to_string(), vec![LetterMatcher::Any(Letter::OpenContextMenu)]));
    categories.push(("Quit".to_string(), vec![LetterMatcher::Any(Letter::ExitSignal)]));

    let mut help_lines = Vec::new();

    for category in categories {
        let mut hl = HelpLine { key_events: Vec::new(), category: category.0 };
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
