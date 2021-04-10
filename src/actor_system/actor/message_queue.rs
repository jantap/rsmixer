use std::collections::VecDeque;

use super::super::{messages::BoxedMessage, Sender};

#[derive(Default)]
pub struct MessageQueue {
	queue: VecDeque<BoxedMessage>,
}

impl MessageQueue {
	pub fn send(&mut self, sx: &Sender<BoxedMessage>) {
		while let Some(msg) = self.queue.pop_front() {
			if let Err(e) = sx.send(msg) {
				self.queue.push_back(e.0);
			}
		}
	}

	pub fn push(&mut self, msg: BoxedMessage) {
		self.queue.push_back(msg);
	}
}
