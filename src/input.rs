use crate::{Action, DISPATCH};

use tokio::{stream::StreamExt, sync::broadcast::Receiver};

pub async fn start(mut rx: Receiver<Action>) {
    let mut reader = crossterm::event::EventStream::new();

    loop {
        let input_event = reader.next();
        let recv_event = rx.next();

        tokio::select! {
            ev = input_event => {
                let ev = if let Some(ev) = ev { ev } else { continue; };
                let ev = if let Ok(ev) = ev { ev } else { continue; };

                match ev {
                    crossterm::event::Event::Key(event) => {
                        DISPATCH.event(Action::KeyPress(event.clone())).await;
                    }
                    crossterm::event::Event::Resize(_, _) => {
                        DISPATCH.event(Action::Redraw).await;
                    }
                    _ => {}
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
