pub mod error;
mod unwind;
mod wasm;

#[cfg(not(feature = "nitro"))]
pub use crate::core::worker_codec::WORKER_UNIX_SOCKET_FD;

use std::{
	collections::{hash_map::Entry, HashMap},
	panic::{catch_unwind, AssertUnwindSafe},
	sync::Arc,
};

use tea_sdk::errorx::SyncResultExt;
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

use crate::{
	core::worker_codec::{read_var_bytes, write_var_bytes, Operation},
	worker::{error::Result, unwind::FutureExt as _, wasm::Host},
};

pub struct Worker {
	read: Mutex<OwnedReadHalf>,
	write: Mutex<OwnedWriteHalf>,
	channels: Mutex<HashMap<u64, Sender<(Operation, u64)>>>,
	host: Host,
}

impl Worker {
	pub async fn init(mut socket: UnixStream) -> Result<Arc<Self>> {
		let is_path = socket.read_u8().await?;
		let host = if is_path == 0 {
			let path = String::from_utf8(read_var_bytes(&mut socket).await?)?;
			let wasm = tokio::fs::read(path).await?;
			Host::new(wasm).await
		} else {
			let wasm = read_var_bytes(&mut socket).await?;
			Host::new(wasm).await
		};
		let (host, metadata) = match host {
			Ok(host) => {
				let metadata = host.metadata().await;
				(Ok(host), Ok(metadata))
			}
			Err(e) => (Err(e.clone()), Err(e)),
		};
		write_var_bytes(&mut socket, &bincode::serialize(&metadata)?).await?;
		socket.flush().await?;
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
					let channel = match channels.entry(cid) {
						Entry::Occupied(entry) => entry.into_mut(),
						Entry::Vacant(entry) => {
							let (tx, rx) = unbounded_channel();
							let tx = entry.insert(tx);
							let channel = self.clone().channel(cid, rx).force_unwind();
							tokio::spawn(async move {
								if let Err(e) = channel.await {
									println!("Worker channel {cid} exits with error: {e:?}");
								}
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

	pub async fn channel(
		self: Arc<Self>,
		cid: u64,
		mut input: Receiver<(Operation, u64)>,
	) -> Result<()> {
		let mut instance = if let Ok(instance) = self.host.create_instance().force_unwind().await {
			instance
		} else {
			self.host.read_new().force_unwind().await?;
			self.host.create_instance().force_unwind().await?
		};
		while let Some((operation, mut gas)) = input.recv().await {
			let resp = {
				let instance = AssertUnwindSafe(&mut instance);
				let operation = AssertUnwindSafe(operation);
				let gas = AssertUnwindSafe(&mut gas);
				catch_unwind(|| {
					let instance = instance;
					let operation = operation;
					let gas = gas;
					instance.0.invoke(operation.0, Some(gas.0))
				})
				.sync_err_into()
				.flatten()
			};
			let resp = match resp {
				Ok(resp) => resp,
				Err(e) => Operation::ReturnErr { error: e.into() },
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
		Ok(())
	}
}
