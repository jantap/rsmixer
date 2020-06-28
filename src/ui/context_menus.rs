use super::{ EntryIdentifier, EntryType };


pub fn context_menu(typ: EntryType) -> Vec<ContextMenuOption> {
    match typ {
        EntryType::Sink => {
            vec![ContextMenuOption::Suspend, ContextMenuOption::Resume, ContextMenuOption::SetAsDefault]
        }
        EntryType::Source => {
            vec![ContextMenuOption::Suspend, ContextMenuOption::Resume, ContextMenuOption::SetAsDefault]
        }
        EntryType::SinkInput => {
            vec![ContextMenuOption::Move, ContextMenuOption::Kill]
        }
        EntryType::SourceOutput => {
            vec![]
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum ContextMenuOption {
    Entry(EntryIdentifier, String),
    Kill,
    Move,
    Suspend,
    Resume,
    SetAsDefault,
}

impl From<ContextMenuOption> for String {
    fn from(option: ContextMenuOption) -> Self {
        match option {
            ContextMenuOption::Entry(_, s) => s.clone(),
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
    NextOptions(Vec<ContextMenuOption>),
    PresentParents,
}

pub fn resolve(ident: EntryIdentifier, answer: ContextMenuOption) -> ContextMenuEffect {
    if answer == ContextMenuOption::Move {
        return ContextMenuEffect::PresentParents;
    }

    return ContextMenuEffect::None;
}
