use std::{any::Any, sync::Arc};

use super::{actor::ActorItem, messages::SystemMessage, Sender};
use crate::prelude::*;

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
    #[allow(dead_code)]
	pub fn stop_actor(&self, id: &'static str) {
		let _ = self
			.internal_sx
			.send(Arc::new(SystemMessage::StopActor(id)));
	}
    #[allow(dead_code)]
	pub fn restart_actor(&self, id: &'static str) {
		let _ = self
			.internal_sx
			.send(Arc::new(SystemMessage::RestartActor(id)));
	}
	pub fn register_actor(&mut self, actor_item: ActorItem) {
		let _ = self
			.internal_sx
			.send(Arc::new(SystemMessage::RegisterActor(actor_item)));
	}
	pub fn start_actor(&self, id: &'static str) {
		let _ = self
			.internal_sx
			.send(Arc::new(SystemMessage::StartActor(id)));
	}
	pub fn actor_panicked(&self, id: &'static str) {
		let _ = self
			.internal_sx
			.send(Arc::new(SystemMessage::ActorTaskFinished(id, None)));
	}
	pub fn actor_returned(&self, id: &'static str, result: Result<()>) {
		let _ = self
			.internal_sx
			.send(Arc::new(SystemMessage::ActorTaskFinished(id, Some(result))));
	}
}
