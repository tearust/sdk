use std::{
	collections::HashMap,
	future::Future,
	sync::{Arc, Weak},
};

use tokio::sync::{
	oneshot::{channel, Receiver},
	RwLock,
};

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
				ids: HashMap::new(),
			})),
		}
	}

	pub(crate) fn create_worker(&self, actor: ActorId) -> Arc<WorkerHandle> {
		let (tx, rx) = channel();
		let handle = WorkerHandle {
			actor: actor.clone(),
			table: Arc::downgrade(&self.table),
			id: Arc::new(RwLock::new(Err(rx))),
		};
		let table = self.table.clone();
		tokio::spawn(async move {
			let mut table = table.write().await;
			let id = match table.ids.entry(actor.clone()) {
				std::collections::hash_map::Entry::Occupied(id) => {
					let id = id.into_mut();
					*id += 1;
					*id
				}
				std::collections::hash_map::Entry::Vacant(id) => *id.insert(0),
			};
			table.workers.insert(
				(actor, id),
				WorkerInfo {
					channels: HashMap::new(),
				},
			);
			_ = tx.send(id);
		});
		Arc::new(handle)
	}

	pub async fn capture(&self) -> HashMap<(ActorId, u64), HashMap<u64, CallingStack>> {
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
	workers: HashMap<(ActorId, u64), WorkerInfo>,
	ids: HashMap<ActorId, u64>,
}

struct WorkerInfo {
	channels: HashMap<u64, Arc<RwLock<CallingStack>>>,
}

pub(crate) struct WorkerHandle {
	actor: ActorId,
	table: Weak<RwLock<WorkerTable>>,
	id: Arc<RwLock<Result<u64, Receiver<u64>>>>,
}

impl WorkerHandle {
	fn id(&self) -> impl Future<Output = u64> + Send + 'static {
		let id = self.id.clone();
		async move {
			let id_lock = id.read().await;
			if let Ok(id) = id_lock.as_ref() {
				return *id;
			}
			drop(id_lock);
			let mut id_lock = id.write().await;
			match id_lock.as_mut() {
				Ok(id) => *id,
				Err(rx) => {
					let id = rx
						.await
						.expect("internal error: worker handle cannot receive its id");
					*id_lock = Ok(id);
					id
				}
			}
		}
	}

	pub fn create_channel(self: &Arc<Self>, cid: u64) -> ChannelHandle {
		let handle = ChannelHandle {
			worker: self.clone(),
			cid,
		};
		if let Some(table) = self.table.upgrade() {
			let actor = self.actor.clone();
			let stack = full_stack().expect("internal error: full_stack not exist");
			let id = self.clone().id();
			tokio::spawn(async move {
				let mut table = table.write().await;
				let id = id.await;
				table
					.workers
					.get_mut(&(actor, id))
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
			let id = self.id();
			tokio::spawn(async move {
				let id = id.await;
				let mut table = table.write().await;
				table.workers.remove(&(actor, id));
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
			let id = self.worker.id();
			let cid = self.cid;
			tokio::spawn(async move {
				let id = id.await;
				let mut table = table.write().await;
				if let Some(worker) = table.workers.get_mut(&(actor, id)) {
					worker.channels.remove(&cid);
				}
			});
		}
	}
}
