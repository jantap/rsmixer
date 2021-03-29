use std::ops::Deref;

use crate::models::{EntryUpdate, RSState};

pub fn handle(msg: &EntryUpdate, state: &mut RSState) {
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
