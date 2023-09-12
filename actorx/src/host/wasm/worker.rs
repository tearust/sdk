use crate::context::host;
use crate::core::{metadata::Metadata, worker_codec::*};
use crate::error::ChannelReceivingTimeout;
use crate::host::OutputHandler;
use crate::ActorId;
use crate::{
	context::{get_gas, set_gas},
	error::{BadWorkerOutput, Error, Result, WorkerCrashed},
};
use std::{
	collections::HashMap,
	env::current_exe,
	fs::Permissions,
	os::unix::prelude::PermissionsExt,
	path::Path,
	process::Stdio,
	sync::{Arc, Weak},
	time::Duration,
};
use tea_sdk::Timeout;
use tokio::fs::set_permissions;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::ChildStdout;
use tokio::{
	fs::OpenOptions,
	io::AsyncWriteExt,
	net::unix::{OwnedReadHalf, OwnedWriteHalf},
	process::{Child, Command},
	sync::{
		mpsc::{unbounded_channel, UnboundedReceiver as Receiver, UnboundedSender as Sender},
		Mutex, RwLock,
	},
};

#[cfg(not(rust_analyzer))]
const WORKER_BINARY: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/worker"));
#[cfg(rust_analyzer)]
const WORKER_BINARY: &[u8] = &[];

pub struct WorkerProcess {
	_proc: Child,
	metadata: Arc<Metadata>,
	write: Mutex<OwnedWriteHalf>,
	read: Mutex<OwnedReadHalf>,
	channels: Mutex<WorkerChannels>,
	#[cfg(feature = "track")]
	tracker: Arc<super::tracker::WorkerHandle>,
	#[cfg(feature = "nitro")]
	handle_path: String,
}

struct WorkerChannels {
	channels: HashMap<u64, Sender<(Operation, u64)>>,
	current_id: u64,
	error: Option<Error>,
}

impl WorkerProcess {
	pub async fn new(source: &[u8], #[cfg(feature = "nitro")] hash: u64) -> Result<Arc<Self>> {
		let path = Self::create_file().await?;

			#[cfg(feature = "nitro")]
			let handle_path;

			let (mut proc, mut this) = {
				let mut cmd = Command::new(path.as_ref());
				cmd.stdout(Stdio::piped()).kill_on_drop(true);
				#[cfg(feature = "nitro")]
				{
					let path = Self::calculate_temp_path(hash);
					let listener = tokio::net::UnixListener::bind(&path)?;
					let mut proc = cmd.stdin(Stdio::piped()).spawn()?;
					let stdin = unsafe { proc.stdin.as_mut().unwrap_unchecked() };
					stdin.write_all(path.as_bytes()).await?;
					stdin.write_u8(b'\n').await?;
					let (this, _) = listener.accept().await?;
					handle_path = path;
					(proc, this)
				}
				#[cfg(not(feature = "nitro"))]
				{
					use command_fds::{tokio::CommandFdAsyncExt, FdMapping};
					use std::os::fd::AsRawFd;
					let (this, other) = tokio::net::UnixStream::pair()?;
					let proc = cmd
						.fd_mappings(vec![FdMapping {
							parent_fd: other.as_raw_fd(),
							child_fd: WORKER_UNIX_SOCKET_FD,
						}])?
						.spawn()?;
					(proc, this)
				}
			};

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
					error: None,
				}),
				#[cfg(feature = "nitro")]
				handle_path,
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

	#[cfg(feature = "nitro")]
	fn calculate_temp_path(hash: u64) -> String {
		use std::{hint::unreachable_unchecked, time::SystemTime};

		let Ok(time) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) else {
			unsafe{ unreachable_unchecked() }
		};
		let time = time.as_millis();

		format!("/tmp/tea-actorx.worker.{hash}.{time}.socket")
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
		const WORKER_PATH: [&str; 3] = [
			".actorx_worker_host.0",
			".actorx_worker_host.1",
			".actorx_worker_host.2",
		];

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

		for i in 0..=2 {
			result.pop();
			result.push(WORKER_PATH[i]);

			if let Ok(true) | Err(_) = tokio::fs::try_exists(&result).await {
				_ = tokio::fs::remove_file(WORKER_PATH[(i + 1) % 3]).await;
				break;
			}
		}

		let mut file = OpenOptions::new()
			.write(true)
			.create(true)
			.open(&result)
			.await?;

		file.write_all(WORKER_BINARY).await?;
		drop(file);

		set_permissions(&result, Permissions::from_mode(0o774)).await?;

		result = tokio::fs::canonicalize(result).await?;
		let result = Arc::from(result);
		*path = Some(Arc::clone(&result));

		info!("Actor host emit worker \"{}\".", result.display());
		Ok(result)
	}

	async fn read_loop(this: Weak<Self>) {
		while let Some(this) = this.upgrade() {
			if let Err(e) = this.read_tick().await {
				info!("worker reader exits with error: {e:?}");
				let mut channels = this.channels.lock().await;
				for (_, sender) in &mut channels.channels.drain() {
					_ = sender.send((Operation::ReturnErr { error: e.clone() }, 0));
				}
				channels.error = Some(e);
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
	pub async fn new(source: &[u8], #[cfg(feature = "nitro")] hash: u64) -> Result<Self> {
		Ok(Self {
			proc: WorkerProcess::new(
				source,
				#[cfg(feature = "nitro")]
				hash,
			)
			.await?,
		})
	}

	pub async fn open(self) -> Result<Channel> {
		let (mut tx, rx) = unbounded_channel();
		let mut channels = self.proc.channels.lock().await;
		if let Some(e) = &channels.error {
			warn!("Worker open returned because of error: {e}");
			return Err(e.clone());
		}
		let mut id = channels.current_id;
		while let Err(last_tx) = channels.channels.try_insert(id, tx) {
			warn!("Channel id {} already exists, try next.", id);
			tx = last_tx.value;
			id = id.wrapping_add(1);
		}
		channels.current_id = id.wrapping_add(1);
		drop(channels);
		Ok(Channel {
			#[cfg(feature = "track")]
			_tracker: self.proc.tracker.create_channel(id),
			proc: self.proc,
			rx,
			id,
		})
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
		let Some((result, gas)) = 
			self.rx.recv().timeout(15000, "invocation").await.map_err(|_| ChannelReceivingTimeout(self.proc.metadata.id.clone()))? 
		 else {
			return Err(WorkerCrashed( self.proc.channels.lock().await.error.as_ref().expect("internal error: worker crashed without error set").clone()).into());
		};
		set_gas(gas);
		Ok(result)
	}

	pub async fn close(self) {
		let mut channels = self.proc.channels.lock().await;
		channels.channels.remove(&self.id);
	}
}

#[cfg(feature = "nitro")]
impl Drop for WorkerProcess {
	fn drop(&mut self) {
		let path = std::mem::take(&mut self.handle_path);
		tokio::spawn(async move {
			_ = tokio::fs::remove_file(path).await;
		});
	}
}
