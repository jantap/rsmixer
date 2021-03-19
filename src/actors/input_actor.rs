use crate::{actor_system::prelude::*, Action};

use tokio::stream::StreamExt;

use crossterm::event::{Event, EventStream, MouseEventKind};

use anyhow::Result;

pub struct InputActor {}

impl InputActor {
    pub fn new() -> Actor {
        Actor::Continous(Box::new(Self {}))
    }

    pub fn blueprint() -> ActorBlueprint {
        ActorBlueprint::new("input", &Self::new)
            .on_panic(|_| -> PinnedClosure { Box::pin(async { true }) })
            .on_error(|_| -> PinnedClosure { Box::pin(async { true }) })
    }
}

#[async_trait]
impl ContinousActor for InputActor {
    async fn start(&mut self, _ctx: Ctx) -> Result<()> {
        Ok(())
    }
    async fn stop(&mut self) {}

    fn run(&mut self, ctx: Ctx, events_rx: MessageReceiver) -> BoxedResultFuture {
        Box::pin(start(events_rx, ctx))
    }
}

pub async fn start(mut rx: MessageReceiver, ctx: Ctx) -> Result<()> {
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
                if ev.is::<Shutdown>() {
                    return Ok(());
                }
            }
        };
    }
}
