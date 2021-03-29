use std::{any::Any, fmt::Debug};

use tokio::task::JoinHandle;

use super::{actor::ActorFactory, actor_entry::ActorEntry, retry_strategy::RetryStrategy};
use crate::prelude::*;

pub type BoxedMessage = Box<dyn Any + Send + Sync + 'static>;

pub trait Message: Any + Send + Sync + Debug {}
impl<T> Message for T where T: Any + Send + Sync + Debug {}

pub enum SystemMessage {
    ActorRegistered(&'static str, ActorFactory, RetryStrategy),
    ActorUpdate(&'static str, ActorEntry),
    SendMsg(&'static str, BoxedMessage),
    RestartActor(&'static str),
    ActorPanicked(&'static str),
    ActorReturnedErr(&'static str, Result<()>),
    UserTask(JoinHandle<()>),
    Shutdown,
    // Broadcast(BoxedMessage),
}

pub struct Shutdown {}
