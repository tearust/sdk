use tea_actorx::worker::{error::Result, Worker};
use tokio::net::UnixStream;

#[tokio::main]
async fn main() -> Result<()> {
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
			let socket = UnixStream::connect(&path).await?;
			tokio::fs::remove_file(path).await?;
			socket
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
	Worker::init(socket).await?.serve().await
}
