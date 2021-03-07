use crate::{pa::{self, common::*}, actor_system::prelude::*, VARIABLES};

use std::time::Duration;

use tokio::{
    stream::StreamExt,
    sync::mpsc,
    task::{self, JoinHandle},
};

use anyhow::Result;

pub struct PulseActor {
    external_sx: mpsc::UnboundedSender<Action>,
    external_rx: Option<mpsc::UnboundedReceiver<Action>>,

    task_handle: Option<JoinHandle<Result<()>>>,
}

impl PulseActor {
    pub fn new() -> BoxedActor {
        let external = mpsc::unbounded_channel::<Action>();
        Box::new(Self {
            external_sx: external.0,
            external_rx: Some(external.1),

            task_handle: None,
        })
    }
}

#[async_trait]
impl Actor for PulseActor {
    async fn start(&mut self, ctx: Ctx) -> Result<()> {
        let rx = self.external_rx.take().unwrap();
        self.task_handle = Some(task::spawn(async move { start_async(rx, ctx).await }));

        Ok(())
    }
    async fn stop(&mut self) {
        if self.task_handle.is_some() {
            let _ = self.external_sx.send(Action::ExitSignal);
            let _ = self.task_handle.take().unwrap().await;
        }
    }
    async fn handle_message(&mut self, _ctx: Ctx, msg: BoxedMessage) -> Result<()> {
        if !msg.is::<Action>() {
            return Ok(());
        }

        let msg = msg.downcast::<Action>().unwrap().as_ref().clone();
        self.external_sx.send(msg)?;

        Ok(())
    }
}

pub async fn start_async(mut external_rx: mpsc::UnboundedReceiver<Action>, ctx: Ctx) -> Result<()> {
    let mut interval = tokio::time::interval(Duration::from_millis(50));

    let send = |ch: &cb_channel::Sender<PAInternal>, msg: PAInternal| -> Result<(), RsError> {
        match ch.send(msg) {
            Ok(()) => Ok(()),
            Err(err) => Err(RsError::ChannelError(err)),
        }
    };
    let retry_time = (*VARIABLES).get().pa_retry_time;

    loop {
        let (info_sx, mut info_rx) = mpsc::unbounded_channel();
        let (internal_actions_sx, mut internal_actions_rx) = mpsc::unbounded_channel();
        let (internal_sx, internal_rx) = cb_channel::unbounded();
        let (pa_finished_sx, mut pa_finished_rx) = mpsc::unbounded_channel();

        let sync_pa = task::spawn_blocking(move || {
            let res = pa::start(internal_rx, info_sx, internal_actions_sx);
            let _ = pa_finished_sx.send(res);
        });

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
                        internal_sx.send(PAInternal::Command(Box::new(cmd.clone())))?;

                        if let Action::ExitSignal = cmd {
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

            let timeout_part = tokio::time::delay_for(std::time::Duration::from_secs(1));
            let event = external_rx.next();
            tokio::select! {
                _ = timeout_part => {},
                ev = event => {
                    if let Some(Action::ExitSignal) = ev {
                        return Ok(());
                    }
                }
            };
        }
    }
}
