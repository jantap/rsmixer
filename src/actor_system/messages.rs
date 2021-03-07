use super::{actor::ActorFactory, actor_entry::ActorEntry};

use std::{any::Any, fmt::Debug};

pub type BoxedMessage = Box<dyn Any + Send + Sync + 'static>;

pub trait Message: Any + Send + Sync + Debug {}
impl<T> Message for T where T: Any + Send + Sync + Debug {}

pub enum SystemMessage {
    ActorRegistered(&'static str, ActorFactory),
    ActorUpdate(&'static str, ActorEntry),
    SendMsg(&'static str, BoxedMessage),
    RestartActor(&'static str),
    Shutdown,
    // Broadcast(BoxedMessage),
}
