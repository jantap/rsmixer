use crate::{Letter, BINDINGS, DISPATCH};

use std::io::Read;

use async_std::prelude::*;
use async_std::task;

use log::info;

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
                // if !event.modifiers.is_empty() {
                //     continue;
                // }
                if let Some(letter) = BINDINGS.get().get(&event.code) {
                    DISPATCH.event(*letter).await;

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
