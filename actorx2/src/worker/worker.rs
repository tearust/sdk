use std::{
	collections::{hash_map::Entry, HashMap},
	sync::Arc,
};

use crate::core::worker_codec::{read_var_bytes, write_var_bytes, Operation};
use crate::worker::{error::Result, wasm::Host};
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
			Host::new(&wasm).await
		} else {
			let wasm = read_var_bytes(&mut socket).await?;
			Host::new(&wasm).await
		};
		let (host, metadata) = match host {
			Ok(host) => {
				let metadata = host.metadata().clone();
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
			match Operation::read(read).await? {
				Ok((operation, cid, gas)) => {
					let mut channels = self.channels.lock().await;
					let channel = match channels.entry(cid) {
						Entry::Occupied(entry) => entry.into_mut(),
						Entry::Vacant(entry) => {
							let (tx, rx) = unbounded_channel();
							let tx = entry.insert(tx);
							let channel = self.clone().channel(cid, rx);
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
		let mut instance = self.host.create_instance()?;
		while let Some((operation, mut gas)) = input.recv().await {
			let resp = instance.invoke(operation, Some(&mut gas)).await;
			let resp = match resp {
				Ok(resp) => resp,
				Err(e) => Operation::ReturnErr { error: e.into() },
			};
			let mut write = self.write.lock().await;
			resp.write(&mut *write, cid, gas).await?;
			write.flush().await?;
		}
		Ok(())
	}
}
