use crate::comms::Letter;
use crate::entry::{Entry, EntryIdentifier, EntryType};
use crate::DISPATCH;

pub fn context_menu(entry: &Entry) -> Vec<ContextMenuOption> {
    match entry.entry_type {
        EntryType::Source | EntryType::Sink => vec![
            if entry.suspended {
                ContextMenuOption::Resume
            } else {
                ContextMenuOption::Suspend
            },
            ContextMenuOption::SetAsDefault,
        ],
        EntryType::SinkInput => vec![ContextMenuOption::Move, ContextMenuOption::Kill],
        EntryType::SourceOutput => vec![],
    }
}

#[derive(PartialEq, Clone)]
pub enum ContextMenuOption {
    MoveToEntry(EntryIdentifier, String),
    Kill,
    Move,
    Suspend,
    Resume,
    SetAsDefault,
}

impl From<ContextMenuOption> for String {
    fn from(option: ContextMenuOption) -> Self {
        match option {
            ContextMenuOption::MoveToEntry(_, s) => s.clone(),
            ContextMenuOption::Kill => "Kill".into(),
            ContextMenuOption::Move => "Move".into(),
            ContextMenuOption::Suspend => "Suspend".into(),
            ContextMenuOption::Resume => "Resume".into(),
            ContextMenuOption::SetAsDefault => "Set as default".into(),
        }
    }
}

pub enum ContextMenuEffect {
    None,
    // NextOptions(Vec<ContextMenuOption>),
    PresentParents,
}

pub async fn resolve(ident: EntryIdentifier, answer: ContextMenuOption) -> ContextMenuEffect {
    match answer {
        ContextMenuOption::Move => ContextMenuEffect::PresentParents,
        ContextMenuOption::MoveToEntry(entry, _) => {
            DISPATCH
                .event(Letter::MoveEntryToParent(ident, entry))
                .await;
            ContextMenuEffect::None
        }
        ContextMenuOption::Suspend => {
            DISPATCH.event(Letter::SetSuspend(ident, true)).await;
            ContextMenuEffect::None
        }
        ContextMenuOption::Resume => {
            DISPATCH.event(Letter::SetSuspend(ident, false)).await;
            ContextMenuEffect::None
        }
        ContextMenuOption::Kill => {
            DISPATCH.event(Letter::KillEntry(ident)).await;
            ContextMenuEffect::None
        }
        _ => ContextMenuEffect::None,
    }
}
