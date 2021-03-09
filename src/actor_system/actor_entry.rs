use super::{
    actor::{Actor, ActorStatus, ActorType},
    context::Ctx,
    messages::BoxedMessage,
    prelude::MessageSender,
    Sender,
};

use std::sync::Arc;

use tokio::{sync::RwLock, task};

pub struct ActorEntry {
    pub actor: Option<Arc<RwLock<Actor>>>,
    pub status: Arc<RwLock<ActorStatus>>,
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
            status: Arc::new(RwLock::new(status)),
            cached_messages: Arc::new(RwLock::new(Vec::new())),
            actor_type,
            event_sx,
        }
    }
}

pub async fn handle_until_queue_empty(
    actor_id: &'static str,
    actor: Arc<RwLock<Actor>>,
    cached_messages: Arc<RwLock<Vec<BoxedMessage>>>,
    status: Arc<RwLock<ActorStatus>>,
    context: Ctx,
) {
    let ctx = context.clone();
    let result = task::spawn(async move {
        {
            let mut status = status.write().await;
            *status = ActorStatus::Handling;
        }

        loop {
            let next_message = {
                let mut cached = cached_messages.write().await;
                if cached.is_empty() {
                    let mut status = status.write().await;
                    *status = ActorStatus::Ready;
                    break;
                }
                cached.remove(0)
            };

            let mut actor = actor.write().await;
            match &mut (*actor) {
                Actor::Eventful(actor) => {
                    actor.handle_message(context.clone(), next_message).await?
                }
                _ => {}
            }
        }

        Ok::<(), anyhow::Error>(())
    })
    .await;

    if result.is_err() {
        ctx.restart_actor(actor_id);
    }
}
