use state::Storage;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::RwLock;
// use async_std::prelude::*;

pub struct Dispatch<T: Send + BishopMessage + 'static>(
    Arc<RwLock<Option<Sender<T>>>>,
    Storage<cb_channel::Sender<T>>,
);
#[derive(Clone)]
pub struct Senders<T: Send + BishopMessage + 'static>(Arc<RwLock<HashMap<u32, Sender<T>>>>);

impl<T: Send + BishopMessage + 'static> Senders<T> {
    pub fn new() -> Self {
        Self {
            0: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, i: u32, sender: Sender<T>) {
        self.0.write().await.insert(i, sender);
    }
}

impl<T: Send + BishopMessage + Clone + std::fmt::Debug + 'static> Dispatch<T> {
    pub fn new() -> Self {
        Self {
            0: Arc::new(RwLock::new(None)),
            1: Storage::new(),
        }
    }

    pub async fn register(&self, sender: Sender<T>, sync_sender: cb_channel::Sender<T>) {
        let mut bis = self.0.write().await;
        *bis = Some(sender);
        self.1.set(sync_sender);

        // unsafe {
        //     let s = self.1.;
        //     *s = Some(sync_sender);
        // };
    }

    pub async fn event(&self, ev: T) {
        match self.0.read().await.as_ref() {
            Some(s) => {
                match s.send(ev.clone()) {
                    _ => {}
                };
            }
            None => {}
        }
    }

    pub fn sync_event(&self, ev: T) {
        match self.1.try_get() {
            Some(s) => {
                match s.send(ev) {
                    Err(_) => {}
                    _ => {}
                };
            }
            None => {}
        }
    }
}

pub trait BishopMessage {
    fn id(&self) -> u32;
}

pub async fn start<T: Send + std::fmt::Debug + Clone + BishopMessage + 'static>(
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
        match s.send(msg) {
            _ => {}
        };
    }
}

#[macro_export]
macro_rules! bishopify_internal {
    ($i:ident) => { _ }
}

#[macro_export]
macro_rules! bishopify {
    ($enu:ident, $( $x:ident $( ( $( $s:ident),* ) )? => $y:expr ),* $( , )?) => {
        use crate::bishopify_internal;

        #[derive(Clone, PartialEq, Debug)]
        pub enum $enu {
            $( $x $( ( $( $s ),* ) )? ),*
        }

        impl BishopMessage for $enu {
            fn id(&self) -> u32 {
                match self {
                    $( $enu::$x $( ( $( bishopify_internal!($s) ),* ) )? => $y ),*
                }
            }
        }

        // bishopify_impl!($enu, $( $x $( ( $( $s),* ) )? => $y ),*);
    }
}
