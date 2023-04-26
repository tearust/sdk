use std::{mem::size_of, sync::Arc};

use crate::core::{
	actor::ActorId,
	metadata::{Claim, Metadata},
	worker_codec::Operation,
};
use tea_codec::serde::{get_type_id, TypeId};
use tokio::{sync::Mutex, task::JoinHandle};

use crate::{
	error::{AccessNotPermitted, Result},
	sdk::{actor::Actor, context::calling_stack, hooks::Deactivate},
};

use self::worker::{Channel, Worker};

pub mod worker;

pub struct WasmActor {
	state: Mutex<State>,
	source: Vec<u8>,
	id: ActorId,
}

struct State {
	current: Worker,
	count: usize,
	loading: Option<JoinHandle<Result<Worker>>>,
}

const MAX_COUNT: usize = 128;

impl WasmActor {
	pub async fn new(wasm_path: &str) -> Result<Self> {
		let mut source = Vec::with_capacity(wasm_path.len() + size_of::<u64>() + 1);
		source.push(0);
		source.extend_from_slice(&(wasm_path.len() as u64).to_le_bytes());
		source.extend_from_slice(wasm_path.as_bytes());
		Self::new_source(source).await
	}

	pub async fn from_binary(wasm_binary: &[u8]) -> Result<Self> {
		let mut source = Vec::with_capacity(wasm_binary.len() + size_of::<u64>() + 1);
		source.push(1);
		source.extend_from_slice(&(wasm_binary.len() as u64).to_le_bytes());
		source.extend_from_slice(wasm_binary);
		Self::new_source(source).await
	}

	async fn new_source(source: Vec<u8>) -> Result<Self> {
		let worker = Worker::new(&source).await?;
		let id = worker.metadata().id.clone();
		Ok(Self {
			state: Mutex::new(State {
				current: worker,
				count: 0,
				loading: None,
			}),
			source,
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
				let source = self.source.clone();
				state.loading = Some(tokio::spawn(async move { Worker::new(&source).await }));
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
					tokio::spawn(channel.close());
					return Ok(resp);
				}

				Operation::ReturnErr { error } => {
					tokio::spawn(channel.close());
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
		let target: ActorId = ctx.into();

		#[allow(clippy::nonminimal_bool)]
		let permitted = (metadata.claims.iter().any(|x| {
			if let Claim::ActorAccess(id) = x {
				&target == id
			} else {
				false
			}
		}) || target == metadata.id)
			&& !(get_type_id(&req) == Ok(Deactivate::TYPE_ID)
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
