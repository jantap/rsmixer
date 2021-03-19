use super::{
    actor::{ActorFactory, ActorStatus, ActorType},
    actor_entry::ActorEntry,
    context::Ctx,
    messages::{BoxedMessage, SystemMessage},
    retry_strategy::RetryStrategy,
    Receiver, Sender, LOGGING_MODULE,
};

use crate::prelude::*;

use std::{collections::HashMap, sync::Arc};

use tokio::{sync::RwLock, task};

use futures::future::{AbortHandle, Abortable};

pub struct Worker {
    entries: HashMap<&'static str, ActorEntry>,
    factories: HashMap<&'static str, ActorFactory>,
    retry_strategies: HashMap<&'static str, RetryStrategy>,
    internal_rx: Arc<RwLock<Receiver<Arc<SystemMessage>>>>,
    internal_sx: Sender<Arc<SystemMessage>>,
    tasks: Vec<AbortHandle>,
}

impl Worker {
    pub fn new(sx: Sender<Arc<SystemMessage>>, rx: Receiver<Arc<SystemMessage>>) -> Self {
        Self {
            entries: HashMap::new(),
            factories: HashMap::new(),
            retry_strategies: HashMap::new(),
            internal_sx: sx,
            internal_rx: Arc::new(RwLock::new(rx)),
            tasks: Vec::new(),
        }
    }

    pub fn start(mut self) -> tokio::task::JoinHandle<Result<()>> {
        let i_rx = Arc::clone(&self.internal_rx);
        task::spawn(async move {
            debug!("Starting worker task");

            let mut i_rx = i_rx.write().await;
            while let Some(msg) = i_rx.recv().await {
                let msg = match Arc::try_unwrap(msg) {
                    Ok(x) => x,
                    Err(_) => {
                        error!("Failed to unwrap Arc<SystemMessage>. Skipping message. This may be very bad news");
                        continue;
                    }
                };
                match msg {
                    SystemMessage::ActorRegistered(id, factory, retry_strategy) => {
                        self.factories.insert(id, factory);
                        self.retry_strategies.insert(id, retry_strategy);
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
                    SystemMessage::ActorPanicked(id) => {
                        self.handle_actor_panic(id);
                    }
                    SystemMessage::ActorReturnedErr(id, result) => {
                        self.handle_actor_returned_err(id, result);
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
        let mut sx = None;
        if let Some(act_entry) = self.entries.get_mut(id) {
            if act_entry.actor.is_some()
                && act_entry.actor_type == Some(ActorType::Eventful)
                && act_entry.status.get().await == ActorStatus::Ready
            {
                if let Some(s) = &act_entry.event_sx {
                    sx = Some(s.clone());
                }
            }
        }
        if let Some(sx) = sx {
            let cached = self.cached_messages_for(id);
            let mut cached = cached.write().await;
            let mut to_send = vec![msg];

            while let Some(c) = cached.pop() {
                to_send.push(c);
            }

            while let Some(c) = to_send.pop() {
                let _ = sx.send(c);
            }
            return;
        }

        let cached = self.cached_messages_for(id);
        let mut cached = cached.write().await;

        cached.push(msg);
    }

    async fn shutdown(&mut self) {
        info!("Shutting down");
        for h in &mut self.tasks {
            h.abort();
        }
        for (_, entry) in &mut self.entries {
            {
                let mut cached = entry.cached_messages.write().await;
                cached.clear();
            }
        }

        for (id, entry) in &mut self.entries {
            super::actor::stop_actor(id, entry).await;
        }
        info!("Finished");
    }

    async fn restart_actor(&mut self, id: &'static str) {
        info!("Restarting actor {}", id);

        if let Some(entry) = self.entries.get_mut(id) {
            super::actor::stop_actor(id, entry).await;
        }

        match self.factories.get(id) {
            Some(factory) => {
                let actor = (factory)();
                super::actor::spawn_actor(self.internal_sx.clone(), id, actor);
            }
            None => {}
        };
    }

    fn handle_actor_panic(&mut self, id: &'static str) {
        if let Some(strategy_orig) = self.retry_strategies.get_mut(id) {
            let ctx: Ctx = self.internal_sx.clone().into();
            let strategy = (strategy_orig.on_panic)(strategy_orig.retry_count);

            strategy_orig.retry_count += 1;

            let (handle, registration) = AbortHandle::new_pair();
            self.tasks.push(handle);

            task::spawn(Abortable::new(
                async move {
                    if strategy.await {
                        ctx.restart_actor(id);
                    }
                },
                registration,
            ));
        }
    }
    fn handle_actor_returned_err(&mut self, id: &'static str, result: Result<()>) {
        if let Some(strategy_orig) = self.retry_strategies.get_mut(id) {
            let ctx: Ctx = self.internal_sx.clone().into();
            let strategy = (strategy_orig.on_error)((strategy_orig.retry_count, result));

            strategy_orig.retry_count += 1;

            let (handle, registration) = AbortHandle::new_pair();
            self.tasks.push(handle);

            task::spawn(Abortable::new(
                async move {
                    if strategy.await {
                        ctx.restart_actor(id);
                    }
                },
                registration,
            ));
        }
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
