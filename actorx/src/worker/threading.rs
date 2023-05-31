use std::{
	any::Any,
	cell::UnsafeCell,
	future::Future,
	mem::transmute,
	panic::{catch_unwind, UnwindSafe},
	pin::Pin,
	sync::{Arc, OnceLock},
	task::Poll,
};

use async_channel as mpmc;
use tea_sdk::errorx::SyncErrorExt;
use tokio::sync::{mpsc, oneshot};

use crate::worker::error::Result;

static GLOBAL_THREAD_POOL: OnceLock<ThreadPool> = OnceLock::new();

pub async fn execute<O>(f: impl FnOnce() -> O + Send + 'static) -> JoinHandle<O>
where
	O: Send + 'static,
{
	GLOBAL_THREAD_POOL
		.get_or_init(ThreadPool::new)
		.execute(f)
		.await
}

pub struct ThreadPool {
	tx: mpmc::Sender<Task>,
}

impl ThreadPool {
	pub fn new() -> Self {
		Self::with_threads(num_cpus::get() + 1)
	}

	pub fn with_threads(n: usize) -> Self {
		assert!(n > 0, "Thread cound have to be greater than 0");
		let (sender, rx) = mpmc::bounded(n);
		for i in 0..n {
			tokio::spawn(daemon(rx.clone(), i));
		}
		Self { tx: sender }
	}

	pub async fn execute<O>(&self, f: impl FnOnce() -> O + Send + 'static) -> JoinHandle<O>
	where
		O: Send + 'static,
	{
		let (tx, rx) = oneshot::channel();
		let (task, result) = {
			let result: Arc<UnsafeCell<Option<O>>> = Arc::new(UnsafeCell::new(None));
			let result_clone = result.clone();
			(
				Task {
					func: unsafe {
						std::mem::transmute(
							Box::new(move || *result_clone.get() = Some(f())) as Box<dyn FnOnce()>
						)
					},
					output: tx,
				},
				unsafe { std::mem::transmute(result) },
			)
		};
		self.tx
			.send(task)
			.await
			.expect("actorx internal error: ThreadPool has no worker thread.");
		JoinHandle(rx, result)
	}
}

pub struct JoinHandle<T>(
	oneshot::Receiver<Result<(), Box<dyn Any + Send>>>,
	Arc<Option<T>>,
);

impl<T> Future for JoinHandle<T> {
	type Output = Result<T>;
	fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
		let slf = self.get_mut();
		match Pin::new(&mut slf.0).poll(cx) {
			Poll::Ready(Ok(Ok(_))) => Poll::Ready(Ok(unsafe {
				&mut *(*(&slf.1 as *const Arc<Option<T>> as *const Arc<UnsafeCell<Option<T>>>))
					.get()
			}
			.take()
			.expect("TaskPool JoinHandle polled after completion"))),
			Poll::Ready(Ok(Err(e))) => Poll::Ready(Err(e.sync_into())),
			Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
			Poll::Pending => Poll::Pending,
		}
	}
}

struct Task {
	func: Box<dyn FnOnce() + Send>,
	output: oneshot::Sender<Result<(), Box<dyn Any + Send>>>,
}

fn spawn_thread(
	n: usize,
) -> mpsc::UnboundedSender<(
	Box<dyn FnOnce() + Send>,
	oneshot::Sender<Result<(), Box<dyn Any + Send>>>,
)> {
	let (tx, rx) = mpsc::unbounded_channel();
	std::thread::Builder::new()
		.name(format!("actorx thread pool {n}"))
		.spawn(|| thread_loop(rx))
		.unwrap();
	tx
}
// Daemon Coroutine
async fn daemon(rx: mpmc::Receiver<Task>, n: usize) {
	let mut tx = spawn_thread(n);

	while let Ok(Task { func, output }) = rx.recv().await {
		let (o, quote) = oneshot::channel();

		if let Err(_) = tx.send((func, o)) {
			_ = output.send(Err(Box::new("thread surprisingly dropped")));
			tx = spawn_thread(n);
			continue;
		}

		let result = quote
			.await
			.map_err(|_| Box::new("thread surprisingly dropped") as _)
			.and_then(|x| x);

		if result.is_err() {
			tx = spawn_thread(n);
		}

		_ = output.send(result);
	}
}

// Worker Thread
fn thread_loop(
	mut rx: mpsc::UnboundedReceiver<(
		Box<dyn FnOnce() + Send>,
		oneshot::Sender<Result<(), Box<dyn Any + Send>>>,
	)>,
) {
	while let Some((input, output)) = rx.blocking_recv() {
		let input: Box<dyn FnOnce() + Send + UnwindSafe> = unsafe { transmute(input) };
		let result = catch_unwind(input);
		_ = output.send(result);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::panic::set_hook;

	#[tokio::test]
	#[allow(unconditional_panic)]
	async fn thread_pool_test() -> Result<()> {
		set_hook(Box::new(|_| {}));
		let pool = ThreadPool::new();
		let mut results = Vec::with_capacity(100);
		for _ in 0..50 {
			results.push(pool.execute(|| 1 / 1).await);
			results.push(pool.execute(|| 1 / 0).await);
		}
		while let Some(handle) = results.pop() {
			let r = handle.await;
			assert!(r.is_err());
			if let Some(handle) = results.pop() {
				let r = handle.await;
				assert_eq!(r, Ok(1));
			}
		}
		println!("all good");
		Ok(())
	}
}
