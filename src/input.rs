use crate::{Action, DISPATCH};

use tokio::{stream::StreamExt, sync::broadcast::Receiver};

use crossterm::event::{Event, EventStream, MouseEventKind};

pub async fn start(mut rx: Receiver<Action>) {
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
                        DISPATCH.event(Action::UserInput(ev)).await;
                    }
                    Event::Mouse(me) => {
                        if MouseEventKind::Moved != me.kind {
                            DISPATCH.event(Action::UserInput(ev)).await;
                        }
                    }
                    Event::Resize(_, _) => {
                        DISPATCH.event(Action::Redraw).await;
                    }
                };
            }
            ev = recv_event => {
                let ev = if let Some(ev) = ev { ev } else { continue; };
                let ev = if let Ok(ev) = ev { ev } else { continue; };
                if ev == Action::ExitSignal {
                    break;
                }
            }
        };
    }
}
