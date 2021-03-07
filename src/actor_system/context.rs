use super::{
    actor::{ActorFactory, ActorStatus},
    actor_entry::ActorEntry,
    messages::SystemMessage,
    Sender,
};

use std::{any::Any, sync::Arc};

#[derive(Clone)]
pub struct Ctx {
    internal_sx: Sender<Arc<SystemMessage>>,
}

impl From<Sender<Arc<SystemMessage>>> for Ctx {
    fn from(internal_sx: Sender<Arc<SystemMessage>>) -> Self {
        Self { internal_sx }
    }
}

impl Ctx {
    pub fn send_to<T: Any + Send + Sync + 'static>(&self, id: &'static str, msg: T) {
        let _ = self
            .internal_sx
            .send(Arc::new(SystemMessage::SendMsg(id, Box::new(msg))));
    }
    pub fn shutdown(&self) {
        let _ = self.internal_sx.send(Arc::new(SystemMessage::Shutdown));
    }
    pub fn restart_actor(&self, id: &'static str) {
        let _ = self
            .internal_sx
            .send(Arc::new(SystemMessage::RestartActor(id)));
    }
    pub fn actor(&mut self, id: &'static str, actor_factory: ActorFactory) {
        let actor = actor_factory();
        let _ = self
            .internal_sx
            .send(Arc::new(SystemMessage::ActorRegistered(id, actor_factory)));
        let _ = self.internal_sx.send(Arc::new(SystemMessage::ActorUpdate(
            id,
            ActorEntry::new(None, ActorStatus::Starting),
        )));

        let sx = self.internal_sx.clone();

        super::actor::spawn_actor(sx, id, actor);
    }
}
