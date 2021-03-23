use crate::{actor_system::Ctx, models::{RSState, EntryUpdate}};

use std::ops::Deref;

pub fn handle(msg: &EntryUpdate, state: &mut RSState, ctx: &Ctx) {
    match msg {
        EntryUpdate::EntryUpdate(ident, entry) => {
            state.update_entry(ident, entry.deref().to_owned());
        }
        EntryUpdate::EntryRemoved(ident) => {
            state.remove_entry(&ident);
        }
        EntryUpdate::PeakVolumeUpdate(ident, peak) => {
            state.update_peak_volume(ident, peak);
        }
    }
}
