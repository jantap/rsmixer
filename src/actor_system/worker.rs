use super::{
    actor::{ActorFactory, ActorStatus, ActorType, Actor},
    actor_entry::{self, ActorEntry},
    error::Error,
    messages::{BoxedMessage, SystemMessage, Shutdown},
    Receiver, Sender,
};

use std::{collections::HashMap, sync::Arc};

use tokio::{sync::RwLock, task};

use anyhow::{Context, Result};

pub struct Worker {
    entries: HashMap<&'static str, ActorEntry>,
    factories: HashMap<&'static str, ActorFactory>,
    internal_rx: Arc<RwLock<Receiver<Arc<SystemMessage>>>>,
    internal_sx: Sender<Arc<SystemMessage>>,
}

impl Worker {
    pub fn new(sx: Sender<Arc<SystemMessage>>, rx: Receiver<Arc<SystemMessage>>) -> Self {
        Self {
            entries: HashMap::new(),
            factories: HashMap::new(),
            internal_sx: sx,
            internal_rx: Arc::new(RwLock::new(rx)),
        }
    }

    pub fn start(mut self) -> tokio::task::JoinHandle<Result<()>> {
        let i_rx = Arc::clone(&self.internal_rx);
        task::spawn(async move {
            let mut i_rx = i_rx.write().await;
            while let Some(msg) = i_rx.recv().await {
                let msg = match Arc::try_unwrap(msg) {
                    Ok(x) => x,
                    Err(_) => {
                        return Err(Error::WrongSystemMessage)
                            .context("When receiving message from Context object");
                    }
                };
                match msg {
                    SystemMessage::ActorRegistered(id, factory) => {
                        self.factories.insert(id, factory);
                    }
                    SystemMessage::ActorUpdate(id, mut actor) => {
                        if let Some(a) = self.entries.get(id) {
                            actor.cached_messages = a.cached_messages.to_owned();
                        }
                        self.entries.insert(id, actor);
                    }
                    SystemMessage::SendMsg(id, m) => {
                        self.send_to(id, m).await;
                    }
                    SystemMessage::RestartActor(id) => {
                        self.restart_actor(id).await;
                    }
                    SystemMessage::Shutdown => {
                        self.shutdown().await;
                        break;
                    }
                };
            }
            Ok(())
        })
    }

    async fn send_to(&mut self, id: &'static str, msg: BoxedMessage) {
        if let Some(entry) = self.entries.get(id) {
            if let Some(ActorType::Continous) = entry.actor_type {
                if let Some(sx) = &entry.event_sx {
                    let _ = sx.send(msg);
                    return;
                }
            }
        }

        self.send_to_eventful(id, msg).await;
    }

    async fn send_to_eventful(&mut self, id: &'static str, msg: BoxedMessage) {
        {
            let cached = self.cached_messages_for(id);
            let mut cached = cached.write().await;

            cached.push(msg);
        }

        if let Some(act_entry) = self.entries.get_mut(id) {
            if act_entry.actor.is_some() && act_entry.actor_type == Some(ActorType::Eventful) {
                let ready = {
                    let status = act_entry.status.read().await;
                    *status == ActorStatus::Ready
                };
                if ready {
                    if let Some(a) = &mut act_entry.actor {
                        let ctx = self.internal_sx.clone().into();
                        let a = Arc::clone(a);
                        let c = Arc::clone(&act_entry.cached_messages);
                        let s = Arc::clone(&act_entry.status);

                        task::spawn(async move {
                            actor_entry::handle_until_queue_empty(id, a, c, s, ctx).await;
                        });
                    }
                }
            }
        }
    }

    async fn shutdown(&mut self) {
        for (_, entry) in &mut self.entries {
            {
                let mut cached = entry.cached_messages.write().await;
                cached.clear();
            }
        }

        for (_, entry) in &mut self.entries {
            super::actor::stop_actor(entry).await;
        }
    }

    async fn restart_actor(&mut self, id: &'static str) {
        if let Some(entry) = self.entries.get_mut(id) {
            super::actor::stop_actor(entry).await;
        }

        match self.factories.get(id) {
            Some(factory) => {
                let actor = (factory)();
                super::actor::spawn_actor(self.internal_sx.clone(), id, actor);
            }
            None => {}
        };
    }

    fn cached_messages_for(&mut self, id: &'static str) -> Arc<RwLock<Vec<BoxedMessage>>> {
        let j = self.entries.get(id).is_some();

        if !j {
            self.entries
                .insert(id, ActorEntry::new(None, ActorStatus::NotRegistered, None));
        }

        Arc::clone(&self.entries.get(id).unwrap().cached_messages)
    }
}
