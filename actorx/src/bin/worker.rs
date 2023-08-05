use std::panic::set_hook;

use tea_actorx::worker::{error::Result, Worker};
use tea_codec::errorx::Global;
use tokio::net::UnixStream;

#[tokio::main]
async fn main() -> Result<()> {
	println!("@@ begin of worker main");
	set_hook(Box::new(|_| {}));
	let socket = {
		#[cfg(feature = "nitro")]
		{
			use tokio::io::{stdin, AsyncBufReadExt, BufReader};
			let mut stdin = BufReader::new(stdin()).lines();
			let path = stdin
				.next_line()
				.await?
				.expect("Internal error: worker stdin is closed");
			drop(stdin);
			UnixStream::connect(&path).await?
		}
		#[cfg(not(feature = "nitro"))]
		{
			use std::os::fd::FromRawFd;
			use tea_actorx::worker::WORKER_UNIX_SOCKET_FD;

			let socket =
				unsafe { std::os::unix::net::UnixStream::from_raw_fd(WORKER_UNIX_SOCKET_FD) };
			socket.set_nonblocking(true)?;
			UnixStream::from_std(socket)?
		}
	};
	if let Err(e) = Worker::init(socket).await?.serve().await {
		if e.name() != Global::StdIo {
			return Err(e);
		}
	}
	println!("@@ end of worker main");
	Ok(())
}
