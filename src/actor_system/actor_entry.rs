use super::{
    actor::{ActorStatus, BoxedActor},
    context::Ctx,
    messages::BoxedMessage,
};

use std::sync::Arc;

use tokio::{sync::RwLock, task};

pub struct ActorEntry {
    pub actor: Option<Arc<RwLock<BoxedActor>>>,
    pub status: Arc<RwLock<ActorStatus>>,
    pub cached_messages: Arc<RwLock<Vec<BoxedMessage>>>,
}

impl ActorEntry {
    pub fn new(actor: Option<BoxedActor>, status: ActorStatus) -> Self {
        Self {
            actor: match actor {
                None => None,
                Some(x) => Some(Arc::new(RwLock::new(x))),
            },
            status: Arc::new(RwLock::new(status)),
            cached_messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

pub async fn handle_until_queue_empty(
    actor_id: &'static str,
    actor: Arc<RwLock<BoxedActor>>,
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
            actor.handle_message(context.clone(), next_message).await?;
        }

        Ok::<(), anyhow::Error>(())
    })
    .await;

    if result.is_err() {
        ctx.restart_actor(actor_id);
    }
}
