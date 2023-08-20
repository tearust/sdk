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

use tea_sdk::ResultExt;
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
	time::Instant,
};

#[cfg(feature = "verbose_log")]
use crate::core::actor::ActorId;
use crate::{
	core::worker_codec::{read_var_bytes, write_var_bytes, Operation},
	worker::{error::Result, wasm::Host},
};

pub struct Worker {
	read: Mutex<OwnedReadHalf>,
	write: Mutex<OwnedWriteHalf>,
	channels: Mutex<HashMap<u64, Sender<(Operation, u64)>>>,
	host: Host,
}

impl Worker {
	pub async fn init(mut socket: UnixStream) -> Result<Arc<Self>> {
		let now = Instant::now();
		let is_path = socket.read_u8().await?;
		let instance_count = socket.read_u8().await?;
		let host = if is_path == 0 {
			let path = String::from_utf8(read_var_bytes(&mut socket).await?)?;
			let wasm = tokio::fs::read(path).await?;
			Host::new(wasm, instance_count).await
		} else {
			let wasm = read_var_bytes(&mut socket).await?;
			Host::new(wasm, instance_count).await
		};
		let (host, metadata) = match host {
			Ok(host) => {
				let metadata = host.metadata();
				(Ok(host), Ok(metadata))
			}
			Err(e) => (Err(e.clone()), Err(e)),
		};
		write_var_bytes(&mut socket, &bincode::serialize(&metadata)?).await?;
		socket.flush().await?;

		let host = host?;
		println!("Worker init done in {:?}", now.elapsed());
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
					let channel = match channels.entry(cid) {
						Entry::Occupied(entry) => entry.into_mut(),
						Entry::Vacant(entry) => {
							let (tx, rx) = unbounded_channel();
							let tx = entry.insert(tx);
							let channel = self.clone().channel(cid, rx);
							let slf = self.clone();
							tokio::spawn(async move {
								if let Err(e) = channel.await {
									let mut write = slf.write.lock().await;
									let writing = match (Operation::ReturnErr { error: e.into() }
										.write(&mut *write, cid, gas)
										.await)
									{
										Ok(_) => write.flush().await.err_into(),
										e => e,
									};
									if let Err(e2) = writing {
										println!("Worker channel {cid} fails, but the error is unable to report due to {e2:?}");
									}
								}
								slf.channels.lock().await.remove(&cid);
							});
							tx
						}
					}
					.clone();
					drop(channels);
					channel
						.send((operation, gas))
						.expect("Actor runtime internal error: worker channel exited");
				}
				Err(i) => unreachable!("Malformed operation {i}"),
			}
		}
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

	pub async fn channel(
		self: Arc<Self>,
		cid: u64,
		mut input: Receiver<(Operation, u64)>,
	) -> Result<()> {
		let state = self.host.get_instance().await;
		let mut first = true;

		while let Some((operation, mut gas)) = input.recv().await {
			let mut state_write = state.write().await;
			let instance = state_write.instance();

			let resp = if first {
				first = false;
				let op = operation.clone();
				match instance.invoke(op, Some(&mut gas)) {
					Ok(r) => r,
					Err(e) => {
						println!("Worker channel fails due to {e:?}, restarting...");
						let mut new_state = self.host.instance_from_cache().await?;
						let result = new_state.instance().invoke(operation, Some(&mut gas))?;
						*state_write = new_state;
						result
					}
				}
			} else {
				instance.invoke(operation, Some(&mut gas))?
			};

			let is_completed = !matches!(resp, Operation::Call { .. });
			if is_completed {
				let slf = self.clone();
				tokio::spawn(async move {
					let mut channels = slf.channels.lock().await;
					channels.remove(&cid);
				});
			}
			let mut write = self.write.lock().await;
			resp.write(&mut *write, cid, gas).await?;
			write.flush().await?;
			if is_completed {
				break;
			}
		}

		state.write().await.reset_idle();
		Ok(())
	}
}
