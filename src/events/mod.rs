mod common;
mod dispatch;
mod events;
mod internal_senders;
mod macros;

pub use dispatch::Dispatch;
pub use events::Letter;
pub use internal_senders::Senders;

use common::*;

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
            let _ = sx.send(m.clone());
        }
    });
    while let Ok(msg) = rx.recv().await {
        if msg.id() == 0 {
            let _ = sync_sx.send(msg.clone());
            let mut senders_inner = Vec::new();
            for (_, sender) in senders.0.read().await.iter() {
                senders_inner.push(sender.clone());
            }
            for s in senders_inner {
                let m = msg.clone();
                let _ = s.send(m);
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
            let _ = s.send(msg);
        });
    }
}
