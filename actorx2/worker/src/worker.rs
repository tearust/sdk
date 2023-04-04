use std::{
	collections::{hash_map::Entry, HashMap},
	sync::Arc,
};

use crate::{error::Result, wasm::Host};
use tea_actorx2_core::worker_codec::{read_var_bytes, write_var_bytes, Operation};
use tokio::{
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
	read: Mutex<WorkerRead>,
	write: Mutex<OwnedWriteHalf>,
	host: Host,
}

struct WorkerRead {
	stream: OwnedReadHalf,
	channels: HashMap<u64, Sender<(Operation, u64)>>,
}

impl Worker {
	pub async fn init(mut socket: UnixStream) -> Result<Arc<Self>> {
		let wasm_path = String::from_utf8(read_var_bytes(&mut socket).await?)?;
		let (host, metadata) = match Host::new(&wasm_path).await {
			Ok(host) => {
				let metadata = host.metadata().clone();
				(Ok(host), Ok(metadata))
			}
			Err(e) => (Err(e.clone()), Err(e)),
		};
		write_var_bytes(&mut socket, &bincode::serialize(&metadata)?).await?;
		let host = host?;
		let (read, write) = socket.into_split();
		Ok(Arc::new(Self {
			host,
			write: Mutex::new(write),
			read: Mutex::new(WorkerRead {
				stream: read,
				channels: HashMap::new(),
			}),
		}))
	}

	pub async fn serve(self: &Arc<Self>) -> Result<()> {
		loop {
			let mut read = self.read.lock().await;
			match Operation::read(&mut read.stream).await? {
				Ok((operation, cid, gas)) => {
					let channel = match read.channels.entry(cid) {
						Entry::Occupied(entry) => entry.into_mut(),
						Entry::Vacant(entry) => {
							let (tx, rx) = unbounded_channel();
							let tx = entry.insert(tx);
							tokio::spawn(self.clone().channel(cid, rx));
							tx
						}
					};
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
			let resp = instance.invoke(operation, Some(&mut gas)).await?;
			let mut write = self.write.lock().await;
			resp.write(&mut *write, cid, gas).await?;
		}
		Ok(())
	}
}
