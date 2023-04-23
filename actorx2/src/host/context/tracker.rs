use std::{
	future::Future,
	mem::swap,
	sync::Arc,
	time::{Duration, SystemTime},
};
use tea_sdk::errorx::Scope;
use tokio::sync::{oneshot, Mutex};

use crate::{
	calling_stack,
	error::{Error, InvocationTimeout},
};

pub struct Tracker {
	state: Mutex<State>,
}

struct State {
	canceller: Option<oneshot::Sender<()>>,
	expiry: SystemTime,
}

impl State {
	fn reset_expriy(&mut self) {
		self.expiry = SystemTime::now() + Duration::from_secs(30);
	}
}

impl Tracker {
	pub const fn new() -> Self {
		Self {
			state: Mutex::const_new(State {
				canceller: None,
				expiry: SystemTime::UNIX_EPOCH,
			}),
		}
	}

	pub async fn track<F, T, S>(self: &Arc<Self>, f: F) -> Result<T, Error<S>>
	where
		F: Future<Output = Result<T, Error<S>>>,
		S: Scope,
	{
		let (tx, rx) = oneshot::channel();
		let mut state = self.state.lock().await;
		let prev_canceller = {
			let mut canceller = Some(tx);
			swap(&mut canceller, &mut state.canceller);
			canceller
		};
		state.reset_expriy();
		drop(state);
		if prev_canceller.is_none() {
			tokio::spawn(self.clone().timer());
		}
		tokio::pin!(f);
		let f2 = &mut f;
		let result = tokio::select! {
			r = f2 => r,
			Ok(_) = rx => Err(InvocationTimeout(calling_stack().expect("internal error: no calling stack")).into()),
			else => f.await
		};

		if prev_canceller.is_none() {
			return result;
		}
		let mut state = self.state.lock().await;
		let is_timeout = state.canceller.is_none();
		state.canceller = prev_canceller;
		state.reset_expriy();
		drop(state);
		if is_timeout {
			tokio::spawn(self.clone().timer());
		}
		result
	}

	async fn timer(self: Arc<Self>) {
		loop {
			let mut state = self.state.lock().await;
			if let Ok(duration) = state.expiry.duration_since(SystemTime::now()) {
				drop(state);
				tokio::time::sleep(duration).await;
			} else {
				let canceller = state
					.canceller
					.take()
					.expect("internal error: canceller token from empty state");
				drop(state);
				_ = canceller.send(());
				return;
			}
		}
	}
}
