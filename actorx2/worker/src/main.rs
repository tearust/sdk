#![feature(min_specialization)]
#![feature(new_uninit)]

extern crate tea_codec as tea_sdk;
#[macro_use]
extern crate tracing;

mod error;
mod wasm;
mod worker;

use error::Result;
use std::os::fd::FromRawFd;
use tea_actorx2_core::worker_codec::*;
use tokio::net::UnixStream;

#[tokio::main]
async fn main() -> Result<()> {
	let socket = unsafe { std::os::unix::net::UnixStream::from_raw_fd(WORKER_UNIX_SOCKET_FD) };
	socket.set_nonblocking(true)?;
	let socket = UnixStream::from_std(socket)?;
	worker::Worker::init(socket).await?.serve().await
}
