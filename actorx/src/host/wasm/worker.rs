use crate::context::host;
use crate::core::{metadata::Metadata, worker_codec::*};
use crate::host::OutputHandler;
use crate::ActorId;
use crate::{
	context::{get_gas, set_gas},
	error::{BadWorkerOutput, Result, WorkerCrashed},
};
use command_fds::{tokio::CommandFdAsyncExt, FdMapping};
use std::env::current_exe;
use std::fs::Permissions;
use std::os::unix::prelude::PermissionsExt;
use std::process::Stdio;
use std::time::Duration;
use std::{
	collections::HashMap,
	os::fd::AsRawFd,
	path::Path,
	sync::{Arc, Weak},
};
use tokio::fs::set_permissions;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::ChildStdout;
use tokio::{
	fs::{remove_file, OpenOptions},
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

const WORKER_BINARY: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/worker"));

pub struct WorkerProcess {
	_proc: Child,
	metadata: Arc<Metadata>,
	write: Mutex<OwnedWriteHalf>,
	read: Mutex<OwnedReadHalf>,
	channels: Mutex<WorkerChannels>,
	#[cfg(feature = "track")]
	tracker: Arc<super::tracker::WorkerHandle>,
}

struct WorkerChannels {
	channels: HashMap<u64, Sender<(Operation, u64)>>,
	current_id: u64,
}

impl WorkerProcess {
	pub async fn new(source: &[u8]) -> Result<Arc<Self>> {
		let path = Self::create_file().await?;

		let (mut this, other) = UnixStream::pair()?;
		let mut proc = Command::new(path.as_ref())
			.stdout(Stdio::piped())
			.kill_on_drop(true)
			.fd_mappings(vec![FdMapping {
				parent_fd: other.as_raw_fd(),
				child_fd: WORKER_UNIX_SOCKET_FD,
			}])?
			.spawn()?;
		drop(other);

		this.write_all(source).await?;
		this.flush().await?;
		let bytes = read_var_bytes(&mut this).await?;
		let metadata: Result<_> = bincode::deserialize(&bytes)?;
		let metadata: Arc<Metadata> = metadata?;

		let (read, write) = this.into_split();

		let out = proc.stdout.take().unwrap();
		let actor = metadata.id.clone();

		let this = Arc::new(Self {
			#[cfg(feature = "track")]
			tracker: crate::context::tracker()?.create_worker(metadata.id.clone()),
			_proc: proc,
			metadata,
			write: Mutex::new(write),
			read: Mutex::new(read),
			channels: Mutex::new(WorkerChannels {
				channels: HashMap::new(),
				current_id: 0,
			}),
		});

		tokio::spawn(Self::redirect_stdout(
			out,
			actor,
			host()?.wasm_output_handler.clone(),
			Arc::downgrade(&this),
		));

		tokio::spawn(Self::read_loop(Arc::downgrade(&this)));

		Ok(this)
	}

	async fn redirect_stdout(
		out: ChildStdout,
		actor: ActorId,
		handler: OutputHandler,
		process: Weak<WorkerProcess>,
	) {
		let mut out = BufReader::new(out).lines();
		while process.strong_count() > 0 {
			let content = tokio::select! {
				Ok(Some(content)) = out.next_line() => content,
				_ = tokio::time::sleep(Duration::from_secs(5)) => continue,
				else => continue,
			};
			let handler = handler.read().await;
			handler(content, actor.clone());
		}
	}

	async fn create_file() -> Result<Arc<Path>> {
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

		let mut result = current_exe()?;
		result.pop();
		result.push(WORKER_PATH);

		info!(
			"Actor host emitting worker executable file as {}",
			result.display()
		);

		_ = remove_file(&result).await;
		let mut file = OpenOptions::new()
			.write(true)
			.create(true)
			.open(&result)
			.await?;
		file.write_all(WORKER_BINARY).await?;
		drop(file);
		set_permissions(&result, Permissions::from_mode(0o755)).await?;
		let result = Arc::from(result);
		*path = Some(Arc::clone(&result));
		Ok(result)
	}

	async fn read_loop(this: Weak<Self>) {
		while let Some(this) = this.upgrade() {
			if let Err(_e) = this.read_tick().await {
				break;
			}
			drop(this);
		}
	}

	async fn read_tick(self: &Arc<Self>) -> Result<()> {
		let mut read = self.read.lock().await;
		let Some(code) = Operation::read_code(&mut *read).await? else {
			return Ok(());
		};
		let input = Operation::read(&mut *read, code).await?;
		drop(read);
		match input {
			Ok((op, cid, gas)) => {
				let channels = self.channels.lock().await;
				let channel = channels
					.channels
					.get(&cid)
					.ok_or_else(|| BadWorkerOutput::ChannelNotExist(cid, self.metadata.id.clone()))?
					.clone();
				drop(channels);
				if let Err(e) = channel.send((op, gas)) {
					warn!("Channel dropped when receiving from worker: {e}");
				}
			}
			Err(code) => {
				return Err(
					BadWorkerOutput::UnknownMasterCommand(code, self.metadata.id.clone()).into(),
				)
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
	pub async fn new(source: &[u8]) -> Result<Self> {
		Ok(Self {
			proc: WorkerProcess::new(source).await?,
		})
	}

	pub async fn open(self) -> Channel {
		let (tx, rx) = unbounded_channel();
		let mut channels = self.proc.channels.lock().await;
		let id = channels.current_id;
		channels.channels.insert(id, tx);
		channels.current_id = channels.current_id.wrapping_add(1);
		drop(channels);
		Channel {
			#[cfg(feature = "track")]
			_tracker: self.proc.tracker.create_channel(id),
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
	#[cfg(feature = "track")]
	_tracker: super::tracker::ChannelHandle,
}

impl Channel {
	pub async fn invoke(&mut self, operation: Operation) -> Result<Operation> {
		let mut write = self.proc.write.lock().await;
		operation.write(&mut *write, self.id, get_gas()).await?;
		write.flush().await?;
		drop(write);
		let (result, gas) = self.rx.recv().await.ok_or(WorkerCrashed)?;
		set_gas(gas);
		Ok(result)
	}

	pub async fn close(self) {
		let mut channels = self.proc.channels.lock().await;
		channels.channels.remove(&self.id);
	}
}
