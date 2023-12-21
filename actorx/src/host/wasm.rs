use std::{mem::size_of, sync::Arc, time::Duration};

use crate::core::{
	actor::ActorId,
	metadata::{Claim, Metadata},
	worker_codec::Operation,
};
use tea_codec::serde::{get_type_id, TypeId};
use tea_sdk::OptionExt;
use tokio::{
	sync::{Mutex, MutexGuard},
	task::JoinHandle,
};

use crate::{
	error::{AccessNotPermitted, Result},
	sdk::{actor::Actor, context::calling_stack, hooks::Deactivate},
};

use self::{
	predictor::VariablePredictor,
	worker::{Channel, Worker},
};

#[cfg(feature = "__test")]
pub use worker::invoke_timeout_ms;

pub mod predictor;
#[cfg(feature = "track")]
pub mod tracker;
pub mod worker;

pub struct WasmActor {
	state: Mutex<State>,
	source: Vec<u8>,
	id: ActorId,
	#[cfg(feature = "nitro")]
	hash: u64,
}

struct State {
	current: Result<Worker, JoinHandle<Result<Worker>>>,
	cached: Option<Result<Worker, JoinHandle<Result<Worker>>>>,
	count: usize,
	instance_count: u8,
	predictor: VariablePredictor,
}

const MAX_COUNT: usize = 128;
const CACHE_COUNT: usize = 100;

impl State {
	fn increase_tick(&mut self) {
		self.count += 1;
		self.predictor.record_request(self.count as u8);
	}

	fn should_reset(&self) -> bool {
		self.count > self.predictor.total
	}

	fn should_cache(&self) -> bool {
		if self.cached.is_some() {
			return false;
		}

		// if the number of instances is less than 3, cache it directly
		if self.count >= MAX_COUNT - 3 {
			return true;
		}

		match self.predictor.predict_time_to_zero() {
			// we assume worker cache instance will be ready in 10 seconds
			Some(duration) => duration < Duration::from_secs(10),
			None => false,
		}
	}
}

impl WasmActor {
	pub async fn new(wasm_path: &str, instance_count: u8, auto_increase: bool) -> Result<Self> {
		let mut source = Vec::with_capacity(wasm_path.len() + size_of::<u64>() + 1);
		source.push(0);
		source.push(instance_count);
		source.push(if auto_increase { 1 } else { 0 });
		source.extend_from_slice(&(wasm_path.len() as u64).to_le_bytes());
		source.extend_from_slice(wasm_path.as_bytes());
		info!("begin of load {wasm_path}");
		Self::new_source(source, instance_count).await
	}

	pub async fn from_binary(
		wasm_binary: &[u8],
		instance_count: u8,
		auto_increase: bool,
	) -> Result<Self> {
		let mut source = Vec::with_capacity(wasm_binary.len() + size_of::<u64>() + 1);
		source.push(1);
		source.push(instance_count);
		source.push(if auto_increase { 1 } else { 0 });
		source.extend_from_slice(&(wasm_binary.len() as u64).to_le_bytes());
		source.extend_from_slice(wasm_binary);
		Self::new_source(source, instance_count).await
	}

	async fn new_source(source: Vec<u8>, instance_count: u8) -> Result<Self> {
		#[cfg(feature = "nitro")]
		let hash = {
			use std::{
				collections::hash_map::DefaultHasher,
				hash::{Hash, Hasher},
			};
			let mut hasher = DefaultHasher::new();
			source.hash(&mut hasher);
			hasher.finish()
		};
		let worker = Worker::new(
			&source,
			#[cfg(feature = "nitro")]
			hash,
		)
		.await?;
		let id = worker.metadata().id.clone();
		Ok(Self {
			state: Mutex::new(State {
				current: Ok(worker),
				cached: None,
				count: 0,
				instance_count,
				predictor: VariablePredictor::new(MAX_COUNT, Duration::from_secs(10)),
			}),
			source,
			id,
			#[cfg(feature = "nitro")]
			hash,
		})
	}

	async fn worker<const INC: bool>(&self) -> Result<Worker> {
		let mut state = self.state.lock().await;

		let result = match &mut state.current {
			Ok(r) => r.clone(),
			Err(task) => {
				let r = match task.await.unwrap() {
					Err(e) => {
						self.new_state(&mut state);
						return Err(e);
					}
					Ok(r) => r,
				};
				state.current = Ok(r.clone());
				r
			}
		};

		if INC {
			state.increase_tick();
			if state.should_reset() {
				info!("reset worker {:?}", result.metadata().id);
				self.new_state(&mut state);
				state.count = 0;
			} else if state.should_cache() {
				info!("cache worker {:?}", result.metadata().id);
				self.new_cache(&mut state);
			}
		}

		Ok(result)
	}

	fn new_cache(&self, state: &mut MutexGuard<State>) {
		let source = self.source.clone();
		#[cfg(feature = "nitro")]
		let hash = self.hash;
		state.cached = Some(Err(crate::spawn(async move {
			Worker::new(
				&source,
				#[cfg(feature = "nitro")]
				hash,
			)
			.await
		})));
	}

	fn new_state(&self, state: &mut MutexGuard<State>) {
		let source = self.source.clone();
		#[cfg(feature = "nitro")]
		let hash = self.hash;

		if let Some(cached) = state.cached.take() {
			state.current = cached;
		} else {
			state.current = Err(crate::spawn(async move {
				Worker::new(
					&source,
					#[cfg(feature = "nitro")]
					hash,
				)
				.await
			}));
		}
	}
}

impl Actor for WasmActor {
	async fn invoke(&self, req: &[u8]) -> Result<Vec<u8>> {
		loop {
			let worker = self.worker::<true>().await?;
			let metadata = worker.metadata().clone();
			let mut channel = match worker.open().await {
				Ok(c) => c,
				Err(e) => {
					warn!("channel error: {e:?}, resetting worker");
					self.state.lock().await.count = MAX_COUNT;
					continue;
				}
			};
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
						Self::inner_call(&mut channel, &metadata, ctx, req).await?
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
	}

	async fn metadata(&self) -> Result<Arc<Metadata>> {
		Ok(self.worker::<false>().await?.metadata().clone())
	}

	fn id(&self) -> Option<ActorId> {
		Some(self.id.clone())
	}

	#[cfg(target_os = "linux")]
	async fn size(&self) -> Result<u64> {
		let worker = self.worker::<false>().await?;
		let pid = worker.pid().ok_or_err("worker pid")?;
		let prc = procfs::process::Process::new(pid as i32)?;
		let process_size = prc.stat()?.rss_bytes();

		// MAX_COUNT / CACHE_COUNT is the approximate coefficient of the number of instances
		let total_size = process_size * MAX_COUNT as u64 / CACHE_COUNT as u64;
		Ok(total_size)
	}

	#[cfg(not(target_os = "linux"))]
	async fn size(&self) -> Result<u64> {
		Ok(0)
	}

	async fn instance_count(&self) -> Result<u8> {
		Ok(self.state.lock().await.instance_count)
	}
}

impl WasmActor {
	#[inline(always)]
	async fn inner_call(
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
