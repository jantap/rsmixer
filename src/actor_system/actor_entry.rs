use super::{
    actor::{Actor, ActorStatus, ActorStatusLocked, ActorType},
    messages::BoxedMessage,
    prelude::MessageSender,
    Sender,
};

use std::sync::Arc;

use tokio::sync::RwLock;

pub struct ActorEntry {
    pub actor: Option<Arc<RwLock<Actor>>>,
    pub status: ActorStatusLocked,
    pub cached_messages: Arc<RwLock<Vec<BoxedMessage>>>,
    pub event_sx: Option<Sender<BoxedMessage>>,
    pub actor_type: Option<ActorType>,
}

impl ActorEntry {
    pub fn new(actor: Option<Actor>, status: ActorStatus, event_sx: Option<MessageSender>) -> Self {
        let actor_type = match &actor {
            Some(actor) => Some(actor.actor_type()),
            None => None,
        };
        Self {
            actor: match actor {
                None => None,
                Some(x) => Some(Arc::new(RwLock::new(x))),
            },
            status: ActorStatusLocked::new(status),
            cached_messages: Arc::new(RwLock::new(Vec::new())),
            actor_type,
            event_sx,
        }
    }
}
