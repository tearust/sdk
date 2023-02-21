use lazy_static::lazy_static;
use std::sync::Arc;

use std::future::Future;
use tokio::{
	runtime::{Builder, EnterGuard, Runtime},
	task::JoinHandle,
};

lazy_static! {
	static ref RUNTIME: Arc<Runtime> = Arc::new(
		Builder::new_multi_thread()
			.enable_all()
			.worker_threads(4)
			// .max_blocking_threads(usize::MAX)
			.build()
			.unwrap()
	);
}

pub fn block_on<T>(f: T) -> T::Output
where
	T: Future,
{
	RUNTIME.block_on(f)
}

pub fn spawn<T>(f: T) -> JoinHandle<T::Output>
where
	T: Future + Send + 'static,
	T::Output: Send + 'static,
{
	RUNTIME.spawn(f)
}

pub fn spawn_blocking<F, R>(f: F) -> JoinHandle<R>
where
	F: FnOnce() -> R + Send + 'static,
	R: Send + 'static,
{
	RUNTIME.spawn_blocking(f)
}

pub fn enter() -> EnterGuard<'static> {
	RUNTIME.enter()
}
