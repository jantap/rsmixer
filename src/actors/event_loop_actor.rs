use crate::{
    action_handlers::*,
    actor_system::prelude::*,
    models::{PageType, RSState, UIMode},
    ui, Action, STYLES,
};

use std::io::Stdout;

use anyhow::Result;

#[derive(Default)]
pub struct EventLoopActor {
    stdout: Option<Stdout>,
    state: RSState,
}

impl EventLoopActor {
    pub fn new() -> Actor {
        Actor::Eventful(Box::new(Self::default()))
    }

    pub fn blueprint() -> ActorBlueprint {
        ActorBlueprint::new("event_loop", &Self::new)
            .on_panic(|_| -> PinnedClosure { Box::pin(async { true }) })
            .on_error(|_| -> PinnedClosure { Box::pin(async { true }) })
    }
}

#[async_trait]
impl EventfulActor for EventLoopActor {
    async fn start(&mut self, ctx: Ctx) -> Result<()> {
        self.stdout = Some(ui::prepare_terminal().unwrap());
        self.state = RSState::default();
        self.state.ui.buffer.set_styles((*STYLES).get().clone());
        self.state.redraw.resize = true;

        ctx.send_to("pulseaudio", Action::RequestPulseAudioState);

        Ok(())
    }

    async fn stop(&mut self) {
        ui::clean_terminal().unwrap();
    }

    async fn handle_message(&mut self, ctx: Ctx, msg: BoxedMessage) -> Result<()> {
        if !msg.is::<Action>() {
            return Ok(());
        }

        let msg = msg.downcast_ref::<Action>().unwrap().clone();

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
