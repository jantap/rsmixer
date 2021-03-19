use crate::{
    actor_system::prelude::*,
    pa::{self, common::*},
    VARIABLES,
};

use std::time::Duration;

use tokio_stream::StreamExt;
use tokio_stream::wrappers::{IntervalStream, UnboundedReceiverStream};
use tokio::{
    sync::mpsc,
    task,
};

use anyhow::Result;

pub struct PulseActor {}

impl PulseActor {
    pub fn new() -> Actor {
        Actor::Continous(Box::new(Self {}))
    }
    pub fn blueprint() -> ActorBlueprint {
        ActorBlueprint::new("pulseaudio", &Self::new)
            .on_panic(|_| -> PinnedClosure { Box::pin(async { true }) })
            .on_error(|_| -> PinnedClosure { Box::pin(async { true }) })
    }
}

#[async_trait]
impl ContinousActor for PulseActor {
    async fn start(&mut self, _ctx: Ctx) -> Result<()> {
        Ok(())
    }
    async fn stop(&mut self) {}
    fn run(&mut self, ctx: Ctx, events_rx: MessageReceiver) -> BoxedResultFuture {
        Box::pin(start_async(events_rx, ctx))
    }
}

async fn start_async(external_rx: MessageReceiver, ctx: Ctx) -> Result<()> {
    let mut interval = IntervalStream::new(tokio::time::interval(Duration::from_millis(50)));

    let send = |ch: &cb_channel::Sender<PAInternal>, msg: PAInternal| -> Result<(), RsError> {
        match ch.send(msg) {
            Ok(()) => Ok(()),
            Err(err) => Err(RsError::ChannelError(err)),
        }
    };
    let retry_time = (*VARIABLES).get().pa_retry_time;
    let mut external_rx = UnboundedReceiverStream::new(external_rx);

    loop {
        let (info_sx, info_rx) = mpsc::unbounded_channel();
        let (internal_actions_sx, internal_actions_rx) = mpsc::unbounded_channel();
        let (internal_sx, internal_rx) = cb_channel::unbounded();
        let (pa_finished_sx, pa_finished_rx) = mpsc::unbounded_channel();

        let sync_pa = task::spawn_blocking(move || {
            let res = pa::start(internal_rx, info_sx, internal_actions_sx);
            let _ = pa_finished_sx.send(res);
        });

        let mut pa_finished_rx = UnboundedReceiverStream::new(pa_finished_rx);
        let mut internal_actions_rx = UnboundedReceiverStream::new(internal_actions_rx);
        let mut info_rx = UnboundedReceiverStream::new(info_rx);

        ctx.send_to("event_loop", Action::ConnectToPulseAudio);

        loop {
            let res = external_rx.next();
            let finished = pa_finished_rx.next();
            let actions = internal_actions_rx.next();
            let info = info_rx.next();
            let timeout = interval.next();

            tokio::select! {
                r = res => {
                    if let Some(cmd) = r {
                        if cmd.is::<Action>() {
                            if let Some(cmd) = cmd.downcast_ref::<Action>() {
                                internal_sx.send(PAInternal::Command(Box::new(cmd.clone())))?;
                            }
                            continue;
                        }
                        if let Some(_) = cmd.downcast_ref::<Shutdown>() {
                            internal_sx.send(PAInternal::Command(Box::new(Action::ExitSignal)))?;
                            sync_pa.await.unwrap();
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
        ctx.send_to("event_loop", Action::PulseAudioDisconnected);
        ctx.send_to("event_loop", Action::PulseAudioDisconnected2);
        for i in 0..retry_time {
            ctx.send_to("event_loop", Action::RetryIn(retry_time - i));

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
