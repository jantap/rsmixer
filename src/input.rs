use crate::{Letter, BINDINGS, DISPATCH};

use tokio::stream::StreamExt;

pub async fn start() {
    let mut reader = crossterm::event::EventStream::new();
    loop {
        let ev = match reader.next().await {
            Some(ev) => ev,
            None => {
                continue;
            }
        };
        let ev = match ev {
            Ok(ev) => ev,
            Err(_) => {
                continue;
            }
        };
        match ev {
            crossterm::event::Event::Key(event) => {
                log::error!("{:?}\n\n{:?}", BINDINGS.get(), event);
                if let Some(letter) = BINDINGS.get().get(&event) {
                    DISPATCH.event(letter.clone()).await;

                    if *letter == Letter::ExitSignal {
                        break;
                    }
                }
            }
            crossterm::event::Event::Resize(_, _) => {
                DISPATCH.event(Letter::Redraw).await;
            }
            _ => {}
        };
    }
}
