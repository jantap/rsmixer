use std::time::Duration;

use anyhow::Result;
use tokio::{sync::mpsc, task};
use tokio_stream::{
	wrappers::{IntervalStream, UnboundedReceiverStream},
	StreamExt,
};

use crate::{
	actor_system::prelude::*,
	models::{EntryUpdate, PAStatus, PulseAudioAction},
	pa::{self, common::*},
	VARIABLES,
};

pub struct PulseActor {}

impl PulseActor {
	pub fn factory() -> Actor {
		Actor::Continous(Box::new(Self {}))
	}
	pub fn item() -> ActorItem {
		ActorItem::new("pulseaudio", &Self::factory)
			.on_panic(|_| -> PinnedClosure { Box::pin(async { true }) })
			.on_error(|_| -> PinnedClosure { Box::pin(async { true }) })
	}
}

#[async_trait]
impl ContinousActor for PulseActor {
	async fn start(&mut self, _ctx: Ctx) {}
	async fn stop(&mut self) {}
	fn run(&mut self, ctx: Ctx, events_rx: LockedReceiver) -> BoxedResultFuture {
		Box::pin(start_async(events_rx, ctx))
	}
}

async fn start_async(external_rx: LockedReceiver, ctx: Ctx) -> Result<()> {
	let mut interval = IntervalStream::new(tokio::time::interval(Duration::from_millis(50)));

	let send = |ch: &cb_channel::Sender<PAInternal>, msg: PAInternal| -> Result<()> {
		match ch.send(msg) {
			Ok(()) => Ok(()),
			Err(err) => Err(PAError::ChannelError(err).into()),
		}
	};
	let retry_time = (*VARIABLES).get().pa_retry_time;
	let mut external_rx = external_rx.write().await;

	loop {
		let (info_sx, info_rx) = mpsc::unbounded_channel();
		let (internal_actions_sx, internal_actions_rx) = mpsc::unbounded_channel::<EntryUpdate>();
		let (internal_sx, internal_rx) = cb_channel::unbounded();
		let (pa_finished_sx, pa_finished_rx) = mpsc::unbounded_channel();

		let sync_pa = task::spawn_blocking(move || {
			let res = pa::start(internal_rx, info_sx, internal_actions_sx);
			let _ = pa_finished_sx.send(res);
		});

		let mut pa_finished_rx = UnboundedReceiverStream::new(pa_finished_rx);
		let mut internal_actions_rx = UnboundedReceiverStream::new(internal_actions_rx);
		let mut info_rx = UnboundedReceiverStream::new(info_rx);

		ctx.send_to("event_loop", PAStatus::ConnectToPulseAudio);

		loop {
			let res = external_rx.next();
			let finished = pa_finished_rx.next();
			let actions = internal_actions_rx.next();
			let info = info_rx.next();
			let timeout = interval.next();

			tokio::select! {
				r = res => {
					if let Some(cmd) = r {
						if cmd.is::<PulseAudioAction>() {
							if let Some(cmd) = cmd.downcast_ref::<PulseAudioAction>() {
								internal_sx.send(PAInternal::Command(Box::new(cmd.clone())))?;
							}
							continue;
						}
						if cmd.downcast_ref::<Shutdown>().is_some() {
							internal_sx.send(PAInternal::Command(Box::new(PulseAudioAction::Shutdown)))?;
							log::error!("starting await");
							sync_pa.await.unwrap();
							log::error!("ending await");
							return Ok(());
						}
					}
				}
				_ = finished => {
					break;
				}
				i = actions => {
					if let Some(action) = i {
						ctx.send_to("event_loop", action);
					}
				}
				i = info => {
					if let Some(ident) = i {
						send(&internal_sx, PAInternal::AskInfo(ident))?;
					}
				}
				_ = timeout => {
					send(&internal_sx, PAInternal::Tick)?;
				}
			};
		}
		ctx.send_to("event_loop", PAStatus::PulseAudioDisconnected);
		for i in 0..retry_time {
			ctx.send_to("event_loop", PAStatus::RetryIn(retry_time - i));

			let timeout_part = tokio::time::sleep(std::time::Duration::from_secs(1));
			let event = external_rx.next();
			tokio::select! {
				_ = timeout_part => {},
				ev = event => {
					if let Some(x) = ev {
						if x.is::<Shutdown>() {
							return Ok(());
						}
					}
				}
			};
		}
	}
}
