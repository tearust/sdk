use std::{
	any::Any,
	cell::UnsafeCell,
	future::{poll_fn, Future},
	mem::transmute,
	panic::{catch_unwind, UnwindSafe},
	pin::Pin,
	sync::{
		atomic::{fence, AtomicBool, Ordering},
		Arc,
	},
	task::Poll,
	thread::{self},
};

use async_channel as mpmc;
use futures::task::AtomicWaker;
use tea_sdk::errorx::SyncErrorExt;
use tokio::sync::oneshot;

use crate::worker::error::Result;

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
		let result = Arc::new(UnsafeCell::new(None));
		let result_clone: Arc<()> = unsafe { transmute(result.clone()) };
		let (tx, rx) = oneshot::channel();
		self.tx
			.send(Task {
				func: Box::new(move || unsafe {
					let result: Arc<UnsafeCell<Option<O>>> = transmute(result_clone);
					*result.get() = Some(f())
				}),
				output: tx,
			})
			.await
			.expect("actorx internal error: ThreadPool has no worker thread.");
		JoinHandle(rx, result)
	}
}

pub struct JoinHandle<T>(
	oneshot::Receiver<Result<(), Box<dyn Any + Send>>>,
	Arc<UnsafeCell<Option<T>>>,
);

impl<T> Future for JoinHandle<T> {
	type Output = Result<T>;
	fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
		let slf = self.get_mut();
		match Pin::new(&mut slf.0).poll(cx) {
			Poll::Ready(Ok(Ok(_))) => Poll::Ready(Ok(unsafe { &mut *slf.1.get() }
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

struct Context {
	state: UnsafeCell<State>,
	task_executing: AtomicBool,
	is_running: AtomicBool,
	waker: AtomicWaker,
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

#[derive(Default)]
enum State {
	#[default]
	OnHold,
	Executing(Box<dyn FnOnce() + Send>),
	Finished(Result<(), Option<Box<dyn Any + Send>>>),
}

// Daemon Coroutine
async fn daemon(rx: mpmc::Receiver<Task>, n: usize) {
	let context = Arc::new(Context {
		state: UnsafeCell::new(State::OnHold),
		task_executing: AtomicBool::new(false),
		is_running: AtomicBool::new(true),
		waker: AtomicWaker::new(),
	});

	let mut thread = {
		let context = context.clone();
		std::thread::Builder::new()
			.name(format!("actorx thread pool {n}"))
			.spawn(|| thread_loop(context))
			.unwrap()
	};

	while let Ok(Task { func, output }) = rx.recv().await {
		fence(Ordering::Acquire);
		unsafe { *context.state.get() = State::Executing(func) };
		context.task_executing.store(true, Ordering::Release);
		thread.thread().unpark();

		poll_fn(|cx| {
			if context.task_executing.load(Ordering::Relaxed) {
				context.waker.register(cx.waker());
				Poll::Pending
			} else {
				Poll::Ready(())
			}
		})
		.await;

		let State::Finished(result) = std::mem::take(unsafe { &mut *context.state.get() }) else {
			unreachable!("actorx internal error: thread pool result unfinished")
		};

		let result = result.map_err(|e| {
			unsafe { *context.state.get() = State::OnHold };
			context.task_executing.store(false, Ordering::Relaxed);
			thread = {
				let context = context.clone();
				std::thread::Builder::new()
					.name(format!("actorx thread pool {n}"))
					.spawn(|| thread_loop(context))
					.unwrap()
			};
			e.expect("actorx internal error: no error")
		});

		_ = output.send(result);
	}

	context.is_running.store(false, Ordering::Relaxed);
}

// Worker Thread
fn thread_loop(context: Arc<Context>) {
	while context.is_running.load(Ordering::Relaxed) {
		if !context.task_executing.load(Ordering::Acquire) {
			thread::park();
			continue;
		}
		let mut input = State::Finished(Err(None));
		std::mem::swap(&mut input, unsafe { &mut *context.state.get() });
		let State::Executing(input) = input else {
			unreachable!("actorx internal error: task_ready is set while task is not ready");
		};

		let input: Box<dyn FnOnce() + Send + UnwindSafe> = unsafe { transmute(input) };
		let result = catch_unwind(input);
		unsafe { *context.state.get() = State::Finished(result.map_err(Some)) };
		context.task_executing.store(false, Ordering::SeqCst);
		context.waker.wake();
		fence(Ordering::Release);
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
