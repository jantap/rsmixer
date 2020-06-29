use async_std::sync::{Arc, Receiver, RwLock, Sender};
use async_std::task;
use std::collections::HashMap;
// use async_std::prelude::*;

pub struct Dispatch<T: Send + BishopMessage + 'static>(Arc<RwLock<Option<Sender<T>>>>);
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

impl<T: Send + BishopMessage + 'static> Dispatch<T> {
    pub fn new() -> Self {
        Self {
            0: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn register(&self, sender: Sender<T>) {
        let mut bis = self.0.write().await;
        *bis = Some(sender);
    }

    pub async fn event(&self, ev: T) {
        match self.0.read().await.as_ref() {
            Some(s) => {
                s.send(ev).await;
            }
            None => {}
        }
    }
}

pub trait BishopMessage {
    fn id(&self) -> u32;
}

pub async fn start<T: Send + Clone + BishopMessage + 'static>(
    rx: Receiver<T>,
    senders: Senders<T>,
) {
    while let Ok(msg) = rx.recv().await {
        if *&msg.id() == 0 {
            let mut senders_inner = Vec::new();
            for (_, sender) in senders.0.read().await.iter() {
                senders_inner.push(sender.clone());
            }
            for s in senders_inner {
                let m = msg.clone();
                s.send(m).await;
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
        task::spawn(async move {
            s.send(msg).await;
        });
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

        #[derive(Clone, PartialEq)]
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
