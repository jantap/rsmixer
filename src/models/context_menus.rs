use crate::{
    entry::{Entry, EntryIdentifier, EntryType},
    Action, DISPATCH,
};

pub fn context_menu(entry: &Entry) -> Vec<ContextMenuOption> {
    match entry.entry_type {
        EntryType::Source | EntryType::Sink => vec![
            if entry.play_entry.as_ref().unwrap().suspended {
                ContextMenuOption::Resume
            } else {
                ContextMenuOption::Suspend
            },
            ContextMenuOption::SetAsDefault,
        ],
        EntryType::SinkInput => vec![ContextMenuOption::Move, ContextMenuOption::Kill],
        EntryType::SourceOutput => vec![],
        EntryType::Card => entry
            .card_entry
            .as_ref()
            .unwrap()
            .profiles
            .iter()
            .map(|p| ContextMenuOption::ChangeCardProfile(p.name.clone(), p.description.clone()))
            .collect(),
    }
}

#[derive(PartialEq, Clone)]
pub enum ContextMenuOption {
    MoveToEntry(EntryIdentifier, String),
    ChangeCardProfile(String, String),
    Kill,
    Move,
    Suspend,
    Resume,
    SetAsDefault,
}

impl From<ContextMenuOption> for String {
    fn from(option: ContextMenuOption) -> Self {
        match option {
            ContextMenuOption::MoveToEntry(_, s) => s,
            ContextMenuOption::ChangeCardProfile(_, s) => s,
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
    MoveEntry,
}

pub async fn resolve(ident: EntryIdentifier, answer: ContextMenuOption) -> ContextMenuEffect {
    match answer {
        ContextMenuOption::Move => ContextMenuEffect::MoveEntry,
        ContextMenuOption::MoveToEntry(entry, _) => {
            DISPATCH
                .event(Action::MoveEntryToParent(ident, entry))
                .await;
            ContextMenuEffect::None
        }
        ContextMenuOption::ChangeCardProfile(name, _) => {
            DISPATCH.event(Action::ChangeCardProfile(ident, name)).await;
            ContextMenuEffect::None
        }
        ContextMenuOption::Suspend => {
            DISPATCH.event(Action::SetSuspend(ident, true)).await;
            ContextMenuEffect::None
        }
        ContextMenuOption::Resume => {
            DISPATCH.event(Action::SetSuspend(ident, false)).await;
            ContextMenuEffect::None
        }
        ContextMenuOption::Kill => {
            DISPATCH.event(Action::KillEntry(ident)).await;
            ContextMenuEffect::None
        }
        _ => ContextMenuEffect::None,
    }
}
