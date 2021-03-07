use crate::{actor_system::prelude::*, Action};

use tokio::stream::StreamExt;

use crossterm::event::{Event, EventStream, MouseEventKind};

use tokio::{
    sync::mpsc,
    task::{self, JoinHandle},
};

use anyhow::Result;

pub struct InputActor {
    external_sx: mpsc::UnboundedSender<Action>,
    external_rx: Option<mpsc::UnboundedReceiver<Action>>,

    task_handle: Option<JoinHandle<Result<()>>>,
}

impl InputActor {
    pub fn new() -> BoxedActor {
        let external = mpsc::unbounded_channel();
        Box::new(Self {
            external_sx: external.0,
            external_rx: Some(external.1),

            task_handle: None,
        })
    }
}

#[async_trait]
impl Actor for InputActor {
    async fn start(&mut self, ctx: Ctx) -> Result<()> {
        let rx = self.external_rx.take().unwrap();
        self.task_handle = Some(task::spawn(async move { start(rx, ctx).await }));

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

pub async fn start(mut rx: mpsc::UnboundedReceiver<Action>, ctx: Ctx) -> Result<()> {
    let mut reader = EventStream::new();

    loop {
        let input_event = reader.next();
        let recv_event = rx.next();

        tokio::select! {
            ev = input_event => {
                let ev = if let Some(ev) = ev { ev } else { continue; };
                let ev = if let Ok(ev) = ev { ev } else { continue; };

                match ev {
                    Event::Key(_) => {
                        ctx.send_to("event_loop",Action::UserInput(ev));
                    }
                    Event::Mouse(me) => {
                        if MouseEventKind::Moved != me.kind {
                            ctx.send_to("event_loop", Action::UserInput(ev));
                        }
                    }
                    Event::Resize(_, _) => {
                        ctx.send_to("event_loop", Action::Redraw);
                    }
                };
            }
            ev = recv_event => {
                let ev = if let Some(ev) = ev { ev } else { continue; };
                if ev == Action::ExitSignal {
                    return Ok(());
                }
            }
        };
    }
}
