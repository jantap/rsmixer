use crate::{
    action_handlers::*,
    actor_system::prelude::*,
    models::{PageType, RSState, UIMode},
    ui, Action, STYLES,
};

use std::io::Stdout;

use anyhow::Result;

#[derive(Default)]
pub struct EventLoop {
    stdout: Option<Stdout>,
    state: RSState,
}

impl EventLoop {
    pub fn new() -> BoxedActor {
        Box::new(Self::default())
    }
}

#[async_trait]
impl Actor for EventLoop {
    async fn start(&mut self, _ctx: Ctx) -> Result<()> {
        self.stdout = Some(ui::prepare_terminal().unwrap());
        self.state = RSState::default();
        self.state.ui.buffer.set_styles((*STYLES).get().clone());
        self.state.redraw.resize = true;

        Ok(())
    }

    async fn stop(&mut self) {
        ui::clean_terminal().unwrap();
    }

    async fn handle_message(&mut self, ctx: Ctx, msg: BoxedMessage) -> Result<()> {
        if !msg.is::<Action>() {
            return Ok(());
        }

        let msg = msg.downcast::<Action>().unwrap().as_ref().clone();

        if let Action::PeakVolumeUpdate(_, _) = msg {
        } else {
            log::debug!("Action: {:#?}", msg);
        }

        match msg {
            Action::ExitSignal => {
                ctx.shutdown();
            }
            Action::UserInput(event) => {
                user_input::action_handler(event, &mut self.state, &ctx).await?;

                if let Some(stdout) = &mut self.stdout {
                    ui::redraw(stdout, &mut self.state).await?;
                }
                self.state.redraw.reset();

                return Ok(());
            }
            _ => {}
        }

        general::action_handler(&msg, &mut self.state, &ctx).await;

        entries_updates::action_handler(&msg, &mut self.state, &ctx).await;

        if self.state.current_page != PageType::Cards {
            play_entries::action_handler(&msg, &mut self.state, &ctx).await;
        }

        match self.state.ui_mode {
            UIMode::Normal => {
                normal::action_handler(&msg, &mut self.state, &ctx).await;
            }
            UIMode::ContextMenu => {
                context_menu::action_handler(&msg, &mut self.state, &ctx).await;
            }
            UIMode::Help => {
                help::action_handler(&msg, &mut self.state, &ctx).await;
            }
            UIMode::MoveEntry(_, _) => {
                move_entry::action_handler(&msg, &mut self.state, &ctx).await;
            }
            _ => {}
        }

        if let Some(stdout) = &mut self.stdout {
            ui::redraw(stdout, &mut self.state).await?;
        }

        self.state.redraw.reset();

        Ok(())
    }
}
