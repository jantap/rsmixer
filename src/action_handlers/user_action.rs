use crate::{
	actor_system::Ctx,
	models::{PageType, PulseAudioAction, RSState, UIMode, UserAction},
};

pub fn handle(msg: &UserAction, state: &mut RSState, ctx: &Ctx) {
	match msg {
		UserAction::MoveUp(how_much) => {
			state.move_up(*how_much as usize);
		}
		UserAction::MoveDown(how_much) => {
			state.move_down(*how_much as usize);
		}
		UserAction::MoveLeft => {
			state.move_left();
		}
		UserAction::MoveRight => {
			state.move_right();
		}
		UserAction::SetSelected(index) => {
			state.set_selected(*index as usize);
		}
		UserAction::ChangePage(page) => {
			state.change_page(*page);
		}
		UserAction::CyclePages(which_way) => {
			ctx.send_to(
				"event_loop",
				UserAction::ChangePage(PageType::from(i8::from(state.current_page) + which_way)),
			);
		}
		UserAction::RequestMute(ident) => {
			if state.ui_mode != UIMode::Normal || state.current_page == PageType::Cards {
				return;
			}

			state.request_mute(ident);
		}
		UserAction::RequstChangeVolume(how_much, ident) => {
			if state.ui_mode != UIMode::Normal || state.current_page == PageType::Cards {
				return;
			}

			state.request_change_volume(*how_much, ident);
		}
		UserAction::OpenContextMenu(ident) => {
			if state.ui_mode == UIMode::Normal {
				state.open_context_menu(ident);
			}
		}
		UserAction::CloseContextMenu => {
			if let UIMode::ContextMenu | UIMode::Help = state.ui_mode {
				state.change_ui_mode(UIMode::Normal);
			}
		}
		UserAction::Confirm => match state.ui_mode {
			UIMode::ContextMenu => {
				state.confirm_context_menu();
			}
			UIMode::MoveEntry(ident, parent) => {
				state.change_ui_mode(UIMode::Normal);
				ctx.send_to(
					"pulseaudio",
					PulseAudioAction::MoveEntryToParent(ident, parent),
				);
			}
			_ => {}
		},
		UserAction::Hide(ident) => {
			if UIMode::Normal == state.ui_mode {
				state.hide_entry(ident);
			}
		}
		UserAction::ShowHelp => {
			if UIMode::Normal == state.ui_mode {
				state.change_ui_mode(UIMode::Help);
			}
		}
		UserAction::RequestQuit => {
			ctx.shutdown();
		}
	}
}
