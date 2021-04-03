use std::collections::HashMap;

use futures::future::{AbortHandle, Abortable};
use tokio::task;

use super::{
	context::Ctx,
	retry_strategy::{PinnedClosure, RetryStrategy},
};
use crate::prelude::*;

#[derive(Default)]
pub struct RetryWorker {
	strategies: HashMap<&'static str, RetryStrategy>,
	running_arbitrers: Vec<AbortHandle>,
}

impl RetryWorker {
	pub fn add_strategy(&mut self, id: &'static str, strategy: RetryStrategy) {
		self.strategies.insert(id, strategy);
	}
	pub fn abort_arbitrers(&mut self) {
		for handle in &mut self.running_arbitrers {
			handle.abort();
		}
	}
	pub fn actor_panic(&mut self, id: &'static str, ctx: Ctx) {
		if let Some(strategy_orig) = self.strategies.get_mut(id) {
			let strategy = (strategy_orig.on_panic)(strategy_orig.retry_count);

			strategy_orig.retry_count += 1;

			self.spawn_arbitrer(id, strategy, ctx);
		}
	}
	pub fn actor_returned_err(&mut self, id: &'static str, result: Result<()>, ctx: Ctx) {
		if let Some(strategy_orig) = self.strategies.get_mut(id) {
			let strategy = (strategy_orig.on_error)((strategy_orig.retry_count, result));

			strategy_orig.retry_count += 1;

			self.spawn_arbitrer(id, strategy, ctx);
		}
	}
	fn spawn_arbitrer(&mut self, id: &'static str, strategy: PinnedClosure, ctx: Ctx) {
		let (handle, registration) = AbortHandle::new_pair();
		self.running_arbitrers.push(handle);

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
