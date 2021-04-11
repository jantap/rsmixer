use std::sync::Arc;

use tokio::sync::RwLock;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ActorStatus {
	Off,
	Ready,
	Stopping,
	Restarting,
	ArbiterRunning,
}

impl ActorStatus {
	pub fn is_off(&self) -> bool {
		matches!(self, Self::ArbiterRunning | Self::Off)
	}
}

#[derive(Clone)]
pub struct LockedActorStatus(Arc<RwLock<ActorStatus>>);

impl LockedActorStatus {
	pub fn new(status: ActorStatus) -> Self {
		Self {
			0: Arc::new(RwLock::new(status)),
		}
	}
	pub async fn set(&self, status: ActorStatus) {
		let mut stat = self.0.write().await;
		*stat = status;
	}
	pub async fn get(&self) -> ActorStatus {
		let stat = self.0.read().await;
		*stat
	}
	pub async fn is_off(&self) -> bool {
		self.get().await.is_off()
	}
}
