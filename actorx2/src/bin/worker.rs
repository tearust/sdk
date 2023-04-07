use std::os::fd::FromRawFd;

use tea_actorx2::worker::{error::Result, Worker, WORKER_UNIX_SOCKET_FD};
use tokio::net::UnixStream;

#[tokio::main]
async fn main() -> Result<()> {
	let socket = unsafe { std::os::unix::net::UnixStream::from_raw_fd(WORKER_UNIX_SOCKET_FD) };
	socket.set_nonblocking(true)?;
	let socket = UnixStream::from_std(socket)?;
	Worker::init(socket).await?.serve().await
}
