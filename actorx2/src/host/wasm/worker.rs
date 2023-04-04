use crate::{
	context::{get_gas, set_gas},
	error::{BadWorkerOutput, Result, WorkerCrashed},
};
use command_fds::{tokio::CommandFdAsyncExt, FdMapping};
use std::{
	collections::HashMap,
	os::fd::AsRawFd,
	path::Path,
	sync::{Arc, Weak},
};
use tea_actorx2_core::{metadata::Metadata, worker_codec::*};
use tokio::{
	fs::{canonicalize, OpenOptions},
	io::AsyncWriteExt,
	net::{
		unix::{OwnedReadHalf, OwnedWriteHalf},
		UnixStream,
	},
	process::{Child, Command},
	sync::{
		mpsc::{unbounded_channel, UnboundedReceiver as Receiver, UnboundedSender as Sender},
		Mutex, RwLock,
	},
};

pub struct WorkerProcess {
	_proc: Child,
	metadata: Arc<Metadata>,
	write: Mutex<OwnedWriteHalf>,
	read: Mutex<WorkerRead>,
}

struct WorkerRead {
	stream: OwnedReadHalf,
	channels: HashMap<u64, Sender<(Operation, u64)>>,
	current_id: u64,
}

impl WorkerProcess {
	pub async fn new(wasm_path: &str) -> Result<Arc<Self>> {
		let path = Self::create_file().await?;

		let (mut this, other) = UnixStream::pair()?;
		let proc = Command::new(path.as_ref())
			.kill_on_drop(true)
			.fd_mappings(vec![FdMapping {
				parent_fd: other.as_raw_fd(),
				child_fd: WORKER_UNIX_SOCKET_FD,
			}])?
			.spawn()?;
		drop(other);

		write_var_bytes(&mut this, wasm_path.as_bytes()).await?;
		let bytes = read_var_bytes(&mut this).await?;
		let metadata: Result<_> = bincode::deserialize(&bytes)?;
		let metadata = metadata?;

		let (read, write) = this.into_split();

		let this = Arc::new(Self {
			_proc: proc,
			metadata,
			write: Mutex::new(write),
			read: Mutex::new(WorkerRead {
				stream: read,
				channels: HashMap::new(),
				current_id: 0,
			}),
		});

		tokio::spawn(Self::read_loop(Arc::downgrade(&this)));

		Ok(this)
	}

	async fn create_file() -> Result<Arc<Path>> {
		const WORKER_BINARY: &[u8] = include_bytes!(env!("CARGO_BIN_FILE_TEA_ACTORX2_WORKER"));
		const WORKER_PATH: &str = ".actorx_worker_host";
		static WORKER_REAL_PATH: RwLock<Option<Arc<Path>>> = RwLock::const_new(None);

		let path = WORKER_REAL_PATH.read().await;
		if let Some(path) = &*path {
			return Ok(path.clone());
		}
		drop(path);
		let mut path = WORKER_REAL_PATH.write().await;
		if let Some(path) = &*path {
			return Ok(path.clone());
		}
		let mut file = OpenOptions::new()
			.mode(0o755)
			.write(true)
			.create(true)
			.open(WORKER_PATH)
			.await?;
		file.write_all(WORKER_BINARY).await?;
		let result = Arc::from(canonicalize(WORKER_PATH).await?);
		*path = Some(Arc::clone(&result));
		Ok(result)
	}

	async fn read_loop(this: Weak<Self>) {
		while let Some(this) = this.upgrade() {
			let mut read = this.read.lock().await;
			if let Err(_e) = Self::read_tick(&mut read, &this.metadata).await {
				break;
			}
		}
	}

	async fn read_tick(read: &mut WorkerRead, metadata: &Metadata) -> Result<()> {
		match Operation::read(&mut read.stream).await? {
			Ok((op, cid, gas)) => {
				let channel = read
					.channels
					.get(&cid)
					.ok_or_else(|| BadWorkerOutput::ChannelNotExist(cid, metadata.id.clone()))?;
				if let Err(e) = channel.send((op, gas)) {
					warn!("Channel dropped when receiving from worker: {e}");
				}
			}
			Err(code) => {
				return Err(BadWorkerOutput::UnknownMasterCommand(code, metadata.id.clone()).into())
			}
		};
		Ok(())
	}
}

#[derive(Clone)]
pub struct Worker {
	proc: Arc<WorkerProcess>,
}

impl Worker {
	pub async fn new(path: &str) -> Result<Self> {
		Ok(Self {
			proc: WorkerProcess::new(path).await?,
		})
	}

	pub async fn open(self) -> Channel {
		let (tx, rx) = unbounded_channel();
		let mut read = self.proc.read.lock().await;
		let id = read.current_id;
		read.channels.insert(id, tx);
		read.current_id = read.current_id.wrapping_add(1);
		drop(read);
		Channel {
			proc: self.proc,
			rx,
			id,
		}
	}

	pub fn metadata(&self) -> &Arc<Metadata> {
		&self.proc.metadata
	}
}

pub struct Channel {
	proc: Arc<WorkerProcess>,
	rx: Receiver<(Operation, u64)>,
	id: u64,
}

impl Channel {
	pub async fn invoke(&mut self, operation: Operation) -> Result<Operation> {
		let mut write = self.proc.write.lock().await;
		operation.write(&mut *write, self.id, get_gas()?).await?;
		drop(write);
		let (result, gas) = self.rx.recv().await.ok_or(WorkerCrashed)?;
		set_gas(gas)?;
		Ok(result)
	}
}
