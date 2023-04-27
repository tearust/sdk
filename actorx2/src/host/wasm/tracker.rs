use std::{
	collections::HashMap,
	sync::{Arc, Weak},
};

use tokio::sync::RwLock;

use crate::{context::full_stack, ActorId, CallingStack};

#[derive(Clone)]
pub struct WorkerTracker {
	table: Arc<RwLock<WorkerTable>>,
}

impl WorkerTracker {
	pub(crate) fn new() -> Self {
		Self {
			table: Arc::new(RwLock::new(WorkerTable {
				workers: HashMap::new(),
			})),
		}
	}

	pub(crate) fn create_worker(&self, actor: ActorId) -> Arc<WorkerHandle> {
		let handle = WorkerHandle {
			actor: actor.clone(),
			table: Arc::downgrade(&self.table),
		};
		let table = self.table.clone();
		tokio::spawn(async move {
			let mut table = table.write().await;
			table.workers.insert(
				actor,
				WorkerInfo {
					channels: HashMap::new(),
				},
			)
		});
		Arc::new(handle)
	}

	pub async fn capture(&self) -> HashMap<ActorId, HashMap<u64, CallingStack>> {
		let table = self.table.read().await;
		let mut result = HashMap::with_capacity(table.workers.capacity());
		for (actor, channels) in &table.workers {
			let mut c = HashMap::with_capacity(channels.channels.capacity());
			for (cid, stack) in &channels.channels {
				c.insert(*cid, stack.read().await.clone());
			}
			result.insert(actor.clone(), c);
		}
		result
	}
}

struct WorkerTable {
	workers: HashMap<ActorId, WorkerInfo>,
}

struct WorkerInfo {
	channels: HashMap<u64, Arc<RwLock<CallingStack>>>,
}

pub(crate) struct WorkerHandle {
	actor: ActorId,
	table: Weak<RwLock<WorkerTable>>,
}

impl WorkerHandle {
	pub fn create_channel(self: &Arc<Self>, cid: u64) -> ChannelHandle {
		let handle = ChannelHandle {
			worker: self.clone(),
			cid,
		};
		if let Some(table) = self.table.upgrade() {
			let actor = self.actor.clone();
			let stack = full_stack().expect("internal error: full_stack not exist");
			tokio::spawn(async move {
				let mut table = table.write().await;
				table
					.workers
					.get_mut(&actor)
					.expect("internal error: worker not exist")
					.channels
					.insert(cid, stack);
			});
		}
		handle
	}
}

impl Drop for WorkerHandle {
	fn drop(&mut self) {
		if let Some(table) = self.table.upgrade() {
			let actor = self.actor.clone();
			tokio::spawn(async move {
				let mut table = table.write().await;
				table.workers.remove(&actor);
			});
		}
	}
}

pub(crate) struct ChannelHandle {
	worker: Arc<WorkerHandle>,
	cid: u64,
}

impl Drop for ChannelHandle {
	fn drop(&mut self) {
		if let Some(table) = self.worker.table.upgrade() {
			let actor = self.worker.actor.clone();
			let cid = self.cid;
			tokio::spawn(async move {
				let mut table = table.write().await;
				table
					.workers
					.get_mut(&actor)
					.expect("internal error: worker not exist")
					.channels
					.remove(&cid);
			});
		}
	}
}
