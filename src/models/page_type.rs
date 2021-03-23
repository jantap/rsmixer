use super::UIMode;

use crate:: entry::{Entries, Entry, EntryIdentifier, EntryKind, EntryType, HiddenStatus};

use std::{fmt::Display, iter};

#[derive(PartialEq, Clone, Hash, Copy, Debug)]
pub enum PageType {
    Output,
    Input,
    Cards,
}
impl Eq for PageType {}
impl Display for PageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl From<PageType> for i8 {
    fn from(p: PageType) -> i8 {
        match p {
            PageType::Output => 0,
            PageType::Input => 1,
            PageType::Cards => 2,
        }
    }
}
impl From<i8> for PageType {
    fn from(p: i8) -> PageType {
        match p {
            -1 => PageType::Cards,
            0 => PageType::Output,
            1 => PageType::Input,
            2 => PageType::Cards,
            _ => PageType::Output,
        }
    }
}
impl PageType {
    pub fn parent_child_types(&self) -> (EntryType, EntryType) {
        match self {
            Self::Output => (EntryType::Sink, EntryType::SinkInput),
            Self::Input => (EntryType::Source, EntryType::SourceOutput),
            Self::Cards => (EntryType::Card, EntryType::Card),
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            PageType::Output => "Output",
            PageType::Input => "Input",
            PageType::Cards => "Cards",
        }
    }
    pub fn as_styled_string(&self) -> String {
        // let styled_name = |pt: PageType| {
        //     if pt == *self {
        //         get_style("normal.bold").apply(pt.as_str())
        //     } else {
        //         get_style("muted").apply(pt.as_str())
        //     }
        // };

        // let divider = get_style("muted").apply(" / ");

        // format!(
        //     "{}{}{}{}{}",
        //     styled_name(PageType::Output),
        //     divider.clone(),
        //     styled_name(PageType::Input),
        //     divider.clone(),
        //     styled_name(PageType::Cards)
        // )
        "".to_string()
    }
    pub fn generate_page<'a>(
        &'a self,
        entries: &'a Entries,
        ui_mode: &'a UIMode,
    ) -> Box<dyn Iterator<Item = (&EntryIdentifier, &Entry)> + 'a> {
        if *self == PageType::Cards {
            return Box::new(entries.iter_type(EntryType::Card));
        }

        let (parent, child) = self.parent_child_types();

        if let UIMode::MoveEntry(ident, parent) = ui_mode {
            let en = entries.get(ident).unwrap();
            let p = *parent;
            let parent_pos = entries
                .iter_type(parent.entry_type)
                .position(|(&i, _)| i == p)
                .unwrap();
            return Box::new(
                entries
                    .iter_type(parent.entry_type)
                    .take(parent_pos + 1)
                    .chain(iter::once((ident, en)))
                    .chain(
                        entries
                            .iter_type(parent.entry_type)
                            .skip(parent_pos + 1)
                            .take_while(|_| true),
                    ),
            );
        }

        Box::new(
            entries
                .iter_type(parent)
                .map(move |(ident, entry)| {
                    std::iter::once((ident, entry)).chain(entries.iter_type(child).filter(
                        move |(_, e)| {
                            e.parent() == Some(ident.index)
                                && match &e.entry_kind {
                                    EntryKind::CardEntry(_) => true,
                                    EntryKind::PlayEntry(play) => {
                                        play.hidden != HiddenStatus::Hidden
                                    }
                                }
                        },
                    ))
                })
                .flatten(),
        )
    }
}
