mod action_handlers;

use action_handlers::*;

use crate::{
    models::{RSState, RedrawType, UIMode},
    ui, Action, RSError,
};

use tokio::{stream::StreamExt, sync::broadcast::Receiver};

pub async fn event_loop(mut rx: Receiver<Action>) -> Result<(), RSError> {
    let mut stdout = ui::prepare_terminal()?;

    let mut state = RSState::default();

    ui::draw_page(&mut stdout, &mut state).await?;

    while let Some(Ok(msg)) = rx.next().await {
        // run action handlers which will decide what to redraw

        log::debug!("Action: {:#?}", msg);

        match msg {
            Action::ExitSignal => {
                break;
            }
            Action::KeyPress(key_event) => {
                key_press::action_handler(key_event, &mut state).await;
                continue;
            }
            _ => {}
        }

        state.redraw = general::action_handler(&msg, &mut state).await;

        entries_updates::action_handler(&msg, &mut state)
            .await
            .apply(&mut state.redraw);

        match state.ui_mode {
            UIMode::Normal => {
                normal::action_handler(&msg, &mut state)
                    .await
                    .apply(&mut state.redraw);
            }
            UIMode::ContextMenu => {
                context_menu::action_handler(&msg, &mut state)
                    .await
                    .apply(&mut state.redraw);
            }
            UIMode::Help => {
                if msg == Action::Redraw {
                    state.redraw.take_bigger(RedrawType::Help);
                }
            }
            UIMode::MoveEntry(_, _) => {
                move_entry::action_handler(&msg, &mut state)
                    .await
                    .apply(&mut state.redraw);
            }
            UIMode::InputVolumeValue => {
                input_volume::action_handler(&msg, &mut state)
                    .await
                    .apply(&mut state.redraw);
            }
            _ => {}
        };

        scroll::scroll_handler(&msg, &mut state)
            .await?
            .apply(&mut state.redraw);

        ui::redraw(&mut stdout, &mut state).await?;
    }
    Ok(())
}
