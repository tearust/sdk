use std::sync::Arc;

use tea_actorx2_core::{
	actor::ActorId,
	metadata::{Claim, Metadata},
	worker_codec::Operation,
};
use tea_codec::serde::{get_type_id, TypeId};
use tokio::{sync::Mutex, task::JoinHandle};

use crate::{
	actor::Actor,
	context::calling_stack,
	error::{AccessNotPermitted, Result},
	hooks::Deactivate,
	invoke::ActorIdExt,
};

use self::worker::{Channel, Worker};

pub mod worker;

pub struct WasmActor {
	state: Mutex<State>,
	wasm_path: String,
	id: ActorId,
}

struct State {
	current: Worker,
	count: usize,
	loading: Option<JoinHandle<Result<Worker>>>,
}

const MAX_COUNT: usize = 128;

impl WasmActor {
	pub async fn new(wasm_path: String) -> Result<Self> {
		let worker = Worker::new(&wasm_path).await?;
		let id = worker.metadata().id.clone();
		Ok(Self {
			state: Mutex::new(State {
				current: worker,
				count: 0,
				loading: None,
			}),
			wasm_path,
			id,
		})
	}

	async fn worker<const INC: bool>(&self) -> Result<Worker> {
		let mut state = self.state.lock().await;

		if state.count == 0 {
			if let Some(r) = state.loading.as_mut() {
				state.current = r.await.unwrap()?;
			}
		}

		if INC {
			state.count += 1;
			if state.count > MAX_COUNT {
				let path = self.wasm_path.clone();
				state.loading = Some(tokio::spawn(async move { Worker::new(&path).await }));
				state.count = 0;
			}
		}

		Ok(state.current.clone())
	}
}

impl Actor for WasmActor {
	async fn invoke(&self, req: &[u8]) -> Result<Vec<u8>> {
		let worker = self.worker::<true>().await?;
		let metadata = worker.metadata().clone();
		let mut channel = worker.open().await;
		let ctx = calling_stack();
		let mut result = channel
			.invoke(Operation::Call {
				ctx: bincode::serialize(&ctx)?,
				req: req.to_vec(),
			})
			.await?;
		loop {
			result = match result {
				Operation::Call { ctx, req } => {
					Self::host_call(&mut channel, &metadata, ctx, req).await?
				}

				Operation::ReturnOk { resp } => {
					return Ok(resp);
				}

				Operation::ReturnErr { error } => {
					return Err(error.into_scope());
				}
			}
		}
	}

	async fn metadata(&self) -> Result<Arc<Metadata>> {
		Ok(self.worker::<false>().await?.metadata().clone())
	}

	fn id(&self) -> Option<ActorId> {
		Some(self.id.clone())
	}
}

impl WasmActor {
	#[inline(always)]
	async fn host_call(
		channel: &mut Channel,
		metadata: &Metadata,
		ctx: Vec<u8>,
		req: Vec<u8>,
	) -> Result<Operation> {
		let target: ActorId = bincode::deserialize(&ctx)?;

		#[allow(clippy::nonminimal_bool)]
		let permitted = metadata.claims.iter().any(|x| {
			if let Claim::ActorAccess(id) = x {
				&target == id
			} else {
				false
			}
		}) && !(get_type_id(&req) == Ok(Deactivate::TYPE_ID)
			&& !calling_stack()
				.map(|x| x.current == target)
				.unwrap_or(false));

		let resp = if permitted {
			match target.invoke_raw(&req).await {
				Ok(resp) => Operation::ReturnOk { resp },
				Err(e) => Operation::ReturnErr {
					error: e.into_scope(),
				},
			}
		} else {
			Operation::ReturnErr {
				error: AccessNotPermitted(target).into(),
			}
		};

		channel.invoke(resp).await
	}
}
