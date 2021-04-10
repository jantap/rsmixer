use std::{io::Stdout, pin::Pin};

use anyhow::Result;
use futures::Future;

use crate::{
	action_handlers::*,
	actor_system::prelude::*,
	models::{
		EntryUpdate, PAStatus, PulseAudioAction, RSState, ResizeScreen, UserAction, UserInput,
	},
	ui, STYLES,
};

#[derive(Default)]
pub struct EventLoopActor {
	stdout: Option<Stdout>,
	state: RSState,
}

impl EventLoopActor {
	pub fn new() -> Actor {
		Actor::Eventful(Box::new(Self::default()))
	}

	pub fn item() -> ActorItem {
		ActorItem::new("event_loop", &Self::new)
			.on_panic(|_| -> PinnedClosure { Box::pin(async { true }) })
			.on_error(|_| -> PinnedClosure { Box::pin(async { true }) })
	}
}

#[async_trait]
impl EventfulActor for EventLoopActor {
	async fn start(&mut self, ctx: Ctx) {
		self.stdout = Some(ui::prepare_terminal().unwrap());
		self.state = RSState::new(ctx.clone());
		self.state.ui.buffer.set_styles((*STYLES).get().clone());
		self.state.redraw.resize = true;

		ctx.send_to("pulseaudio", PulseAudioAction::RequestPulseAudioState);
	}

	async fn stop(&mut self) {
		ui::clean_terminal().unwrap();
	}

	fn handle_message<'a>(
		&'a mut self,
		ctx: Ctx,
		msg: BoxedMessage,
	) -> Pin<Box<dyn Future<Output = Result<()>> + Send + Sync + 'a>> {
		Box::pin(async move {
			if msg.is::<EntryUpdate>() {
				let msg = msg.downcast_ref::<EntryUpdate>().unwrap();

				pulseaudio_info::handle(&msg, &mut self.state);
			} else if msg.is::<PAStatus>() {
				let msg = msg.downcast_ref::<PAStatus>().unwrap();

				pulseaudio_status::handle(&msg, &mut self.state);
			} else if msg.is::<UserInput>() {
				let msg = msg.downcast_ref::<UserInput>().unwrap();

				user_input::handle(&msg, &self.state, &ctx)?;
			} else if msg.is::<UserAction>() {
				let msg = msg.downcast_ref::<UserAction>().unwrap();

				user_action::handle(&msg, &mut self.state, &ctx);
			} else if msg.is::<ResizeScreen>() {
				self.state.redraw.resize = true;
			}

			if self.state.redraw.anything() {
				if let Some(stdout) = &mut self.stdout {
					ui::redraw(stdout, &mut self.state).await?;
				}

				self.state.redraw.reset();
			}

			Ok(())
		})
	}
}
