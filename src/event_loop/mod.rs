mod action_handlers;

use action_handlers::*;

use crate::{
    models::{PageType, RSState, UIMode},
    ui, Action, RSError, STYLES,
};

use tokio::{stream::StreamExt, sync::broadcast::Receiver};

pub async fn event_loop(mut rx: Receiver<Action>) -> Result<(), RSError> {
    let mut stdout = ui::prepare_terminal()?;

    let mut state = RSState::default();

    state.ui.screen.set_styles((*STYLES).get().clone());

    state.redraw.resize = true;
    state.redraw.full = true;

    while let Some(Ok(msg)) = rx.next().await {
        // run action handlers which will decide what to redraw

        if let Action::PeakVolumeUpdate(_, _) = msg {
        } else {
            log::debug!("Action: {:#?}", msg);
        }

        match msg {
            Action::ExitSignal => {
                break;
            }
            Action::UserInput(event) => {
                user_input::action_handler(event, &mut state).await?;

                ui::redraw(&mut stdout, &mut state).await?;
                state.redraw.reset();

                continue;
            }
            _ => {}
        }

        general::action_handler(&msg, &mut state).await;

        entries_updates::action_handler(&msg, &mut state).await;

        if state.current_page != PageType::Cards {
            play_entries::action_handler(&msg, &mut state).await;
        }

        match state.ui_mode {
            UIMode::Normal => {
                normal::action_handler(&msg, &mut state).await;
            }
            UIMode::ContextMenu => {
                context_menu::action_handler(&msg, &mut state).await;
            }
            UIMode::Help => {
                help::action_handler(&msg, &mut state).await;
            }
            UIMode::MoveEntry(_, _) => {
                move_entry::action_handler(&msg, &mut state).await;
            }
            _ => {}
        }

        ui::redraw(&mut stdout, &mut state).await?;

        state.redraw.reset();
    }
    Ok(())
}
