pub mod error;
mod wasm;

#[cfg(not(feature = "nitro"))]
pub use crate::core::worker_codec::WORKER_UNIX_SOCKET_FD;

use std::{
	collections::{hash_map::Entry, HashMap},
	sync::Arc,
};
#[cfg(feature = "verbose_log")]
use ::{std::time::SystemTime, tea_sdk::serde::get_type_id};

use tea_sdk::IntoGlobal;
use tokio::{
	io::{AsyncReadExt, AsyncWriteExt},
	net::{
		unix::{OwnedReadHalf, OwnedWriteHalf},
		UnixStream,
	},
	sync::{
		mpsc::{unbounded_channel, UnboundedReceiver as Receiver, UnboundedSender as Sender},
		Mutex,
	},
};

#[cfg(feature = "verbose_log")]
use crate::core::actor::ActorId;
use crate::{
	core::worker_codec::{read_var_bytes, write_var_bytes, Operation},
	error::ActorX,
	worker::{
		error::{Error, Result},
		wasm::{get_instance, instance_from_cache, Host},
	},
};

use self::wasm::SharedState;

pub struct Worker {
	read: Mutex<OwnedReadHalf>,
	write: Mutex<OwnedWriteHalf>,
	channels: Mutex<HashMap<u64, Sender<(Operation, u64)>>>,
	host: Host,
}

impl Worker {
	pub async fn init(mut socket: UnixStream) -> Result<Arc<Self>> {
		let is_path = socket.read_u8().await?;
		let instance_count = socket.read_u8().await?;
		let auto_increase = socket.read_u8().await? == 1;
		let host = if is_path == 0 {
			let path = String::from_utf8(read_var_bytes(&mut socket).await?)?;
			let wasm = tokio::fs::read(path).await?;
			Host::new(wasm, instance_count, auto_increase).await
		} else {
			let wasm = read_var_bytes(&mut socket).await?;
			Host::new(wasm, instance_count, auto_increase).await
		};
		let (host, metadata) = match host {
			Ok(host) => {
				let metadata = host.metadata();
				(Ok(host), Ok(metadata))
			}
			Err(e) => {
				let err_msg = format!("{e:?}");
				(
					Err(Error::Unnamed(err_msg.clone())),
					Err(ActorX::WasmWorkerError(err_msg)),
				)
			}
		};
		write_var_bytes(&mut socket, &bincode::serialize(&metadata)?).await?;
		socket.flush().await?;

		if let Ok(metadata) = metadata.as_ref() {
			let proc_id = std::process::id();
			println!(
				"Worker initialized with {instance_count} instances, auto increase: {auto_increase}, actor id: {}, process id: {}",
				metadata.id, proc_id
			);
		}

		let host = host?;
		let (read, write) = socket.into_split();
		Ok(Arc::new(Self {
			host,
			write: Mutex::new(write),
			read: Mutex::new(read),
			channels: Mutex::new(HashMap::new()),
		}))
	}

	pub async fn serve(self: &Arc<Self>) -> Result<()> {
		let mut read = self.read.lock().await;
		loop {
			let read = &mut *read;
			let code = read.read_u8().await?;
			match Operation::read(read, code).await? {
				Ok((operation, cid, gas)) => {
					let mut channels = self.channels.lock().await;
					let (channel, is_new, rx) = match channels.entry(cid) {
						Entry::Occupied(entry) => (entry.into_mut().clone(), false, None),
						Entry::Vacant(entry) => {
							let (tx, rx) = unbounded_channel();
							entry.insert(tx.clone());
							(tx, true, Some(rx))
						}
					};
					drop(channels);

					if is_new {
						self.process_new(cid, gas, rx.expect("channel rx")).await;
					}

					channel
						.send((operation, gas))
						.expect("Actor runtime internal error: worker channel exited");
				}
				Err(i) => unreachable!("Malformed operation {i}"),
			}
		}
	}

	async fn process_new(self: &Arc<Self>, cid: u64, gas: u64, rx: Receiver<(Operation, u64)>) {
		let states = self.host.states();
		let slf = self.clone();
		let auto_increase = self.host.auto_increase();
		let compiled_path = self.host.compiled_path().to_string();
		let metadata = self.host.metadata().clone();

		tokio::spawn(async move {
			let state = get_instance(&states, auto_increase, &compiled_path, metadata).await;
			let channel = slf.clone().channel(cid, state.clone(), rx);
			if let Err(e) = channel.await {
				let mut write = slf.write.lock().await;
				let res = Operation::ReturnErr {
					error: ActorX::WasmWorkerError(format!("{e:?}")),
				}
				.write(&mut *write, cid, gas)
				.await;
				let writing = match res {
					Ok(_) => write.flush().await.into_g(),
					e => e,
				};
				if let Err(e2) = writing {
					println!("Worker channel {cid} fails, but the error is unable to report due to {e2:?}");
				}
			}
			slf.channels.lock().await.remove(&cid);
			state.write().await.reset_idle();
		});
	}

	#[cfg(feature = "verbose_log")]
	fn log_operation(op: &Operation, id: ActorId) -> impl FnOnce(&Operation) {
		let calc_op = |op: &Operation| match op {
			Operation::Call { req, .. } => {
				format!("request {}", get_type_id(req).unwrap_or("[untyped]"))
			}
			Operation::ReturnOk { resp } => {
				format!("response {}", get_type_id(resp).unwrap_or("[untyped]"))
			}
			Operation::ReturnErr { error } => {
				format!("error {:?}", error)
			}
		};
		let req = calc_op(op);
		println!("Wasm worker {id} processing {req}");
		let time = SystemTime::now();
		move |op| {
			let resp = calc_op(op);
			println!(
				"Wasm worker {id} finished processing {req}, resulted in {resp} in {}ms",
				time.elapsed().unwrap().as_millis()
			)
		}
	}

	pub(crate) async fn channel(
		self: Arc<Self>,
		cid: u64,
		state: SharedState,
		mut input: Receiver<(Operation, u64)>,
	) -> Result<()> {
		let mut first = true;
		let mut state_write = state.write().await;

		while let Some((operation, mut gas)) = input.recv().await {
			let resp = if first {
				first = false;
				let op = operation.clone();
				match state_write.instance().invoke(op, Some(&mut gas)) {
					Ok(r) => r,
					Err(e) => {
						println!("Worker channel fails due to {e:?}, restarting...");
						let mut new_state =
							instance_from_cache(self.host.compiled_path(), self.host.metadata())
								.await?;
						let result = new_state.instance().invoke(operation, Some(&mut gas))?;
						*state_write = new_state;
						result
					}
				}
			} else {
				state_write
					.instance()
					.invoke(operation, Some(&mut gas))
					.map_err(|e| {
						println!("Worker channel (cid {cid}) invoke fails due to {e:?}");
						e
					})?
			};

			let is_completed = !matches!(resp, Operation::Call { .. });
			let mut write = self.write.lock().await;
			resp.write(&mut *write, cid, gas).await?;
			write.flush().await?;
			if is_completed {
				break;
			}
		}

		Ok(())
	}
}
