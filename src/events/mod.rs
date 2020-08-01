
mod macros;
mod events;
mod dispatch;
mod internal_senders;
mod common;

use common::*;
pub use internal_senders::Senders;
pub use dispatch::Dispatch;

pub use events::Letter;

pub static CHANNEL_CAPACITY: usize = 32;

pub static UI_MESSAGE: u32 = 1;
pub static PA_MESSAGE: u32 = 2;

pub trait Message {
    fn id(&self) -> u32;
}

pub async fn start<T: Send + std::fmt::Debug + Clone + Message + 'static>(
    sx: Sender<T>,
    mut rx: Receiver<T>,
    sync_rx: cb_channel::Receiver<T>,
    sync_sx: cb_channel::Sender<T>,
    senders: Senders<T>,
) {
    tokio::task::spawn_blocking(move || {
        while let Ok(m) = sync_rx.recv() {
            if m.id() == 0 {
                break;
            }
            match sx.send(m.clone()) {
                _ => {}
            };
        }
    });
    while let Ok(msg) = rx.recv().await {
        if *&msg.id() == 0 {
            match sync_sx.send(msg.clone()) {
                _ => {}
            };
            let mut senders_inner = Vec::new();
            for (_, sender) in senders.0.read().await.iter() {
                senders_inner.push(sender.clone());
            }
            for s in senders_inner {
                let m = msg.clone();
                match s.send(m) {
                    _ => {}
                };
            }
            break;
        }
        let s = match senders.0.read().await.get(&msg.id()) {
            Some(s) => s.clone(),
            None => {
                continue;
            }
        };

        // @TODO przemyśleć to - istnieje możliwość że s będzie dropped zanim wiadomość dojdzie
        //                       czy nie lepiej jest mieć gwarancję że dojdzie?
        tokio::task::spawn(async move {
            match s.send(msg) {
                _ => {}
            };
        });
    }
}
