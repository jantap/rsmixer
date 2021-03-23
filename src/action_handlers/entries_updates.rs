use super::common::*;

use crate::{
    entry::{EntryIdentifier, EntryKind, HiddenStatus},
    ui::util::parent_child_types,
};

use crate::ui::Scrollable;

use std::collections::{HashMap, HashSet};

pub async fn action_handler(msg: &Action, state: &mut RSState, ctx: &Ctx) {
    // we only need to update page entries if entries changed
    match msg {
        Action::Redraw
        | Action::EntryRemoved(_)
        | Action::EntryUpdate(_, _)
        | Action::ChangePage(_) => {}

        Action::Hide(Some(ident)) => {
            state.entries.hide(*ident);
        }
        Action::Hide(None) => {
            if let Some(ident) = state.page_entries.get_selected() {
                state.entries.hide(ident);
            }
        }
        _ => {
            return;
        }
    };

}

