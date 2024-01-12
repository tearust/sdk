use std::{
	future::Future,
	mem::swap,
	sync::Arc,
	time::{Duration, SystemTime},
};
use tea_sdk::{errorx::InvocationTimeout, serialize};
use tokio::sync::{oneshot, Mutex};

use crate::{calling_stack, error::Error};

#[cfg(feature = "track")]
use crate::CallingStack;
#[cfg(feature = "track")]
use tokio::sync::RwLock;

pub struct Tracker {
	state: Mutex<State>,
	#[cfg(feature = "track")]
	full_stack: Arc<RwLock<CallingStack>>,
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
	pub fn new(#[cfg(feature = "track")] full_stack: CallingStack) -> Self {
		Self {
			state: Mutex::const_new(State {
				canceller: None,
				expiry: SystemTime::UNIX_EPOCH,
			}),
			#[cfg(feature = "track")]
			full_stack: Arc::new(RwLock::new(full_stack)),
		}
	}

	#[cfg(feature = "track")]
	#[inline(always)]
	pub fn full_stack(&self) -> &Arc<RwLock<CallingStack>> {
		&self.full_stack
	}

	pub async fn track<F, T>(self: &Arc<Self>, f: F) -> Result<T, Error>
	where
		F: Future<Output = Result<T, Error>>,
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
			Ok(_) = rx => Err(InvocationTimeout(serialize(&calling_stack().expect("internal error: no calling stack"))?).into()),
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::context::CALLING_STACK;
	use crate::{ActorId, IntoActor};
	use futures::stream::FuturesUnordered;
	use futures::StreamExt;
	use tea_sdk::errorx::Global;
	use tokio::sync::oneshot::Receiver;

	async fn new_track(expiry_millis: u64) -> (Arc<Tracker>, Receiver<()>) {
		let test_actor: ActorId = b"test_actor".to_vec().into_actor();
		let current_stack = CallingStack::step(test_actor);
		assert!(current_stack.caller.is_none());

		let (sender, receiver) = oneshot::channel();

		let tracker = Arc::new(Tracker::new(current_stack));
		let mut state = tracker.state.lock().await;
		state.expiry = SystemTime::now() + Duration::from_millis(expiry_millis);
		state.canceller = Some(sender);
		drop(state);

		(tracker, receiver)
	}

	#[tokio::test]
	async fn timer_works() {
		let (tracker, receiver) = new_track(10).await;

		tokio::spawn(tracker.clone().timer());

		let result = tokio::select! {
			Ok(_) = receiver => Ok(()),
			else => Err(())
		};
		assert!(result.is_ok());
	}

	#[tokio::test]
	async fn track_works() {
		let test_actor: ActorId = b"test_actor".to_vec().into_actor();
		let current_stack = CallingStack::step(test_actor);
		assert!(current_stack.caller.is_none());

		let tracker = Arc::new(Tracker::new(current_stack));
		let result = tracker
			.track::<_, _>(async {
				tokio::time::sleep(Duration::from_millis(10)).await;
				Ok(())
			})
			.await;
		assert!(result.is_ok());
	}

	#[tokio::test]
	#[ignore]
	async fn track_timeout_works() {
		let test_actor: ActorId = b"test_actor".to_vec().into_actor();
		let current_stack = CallingStack::step(test_actor);
		assert!(current_stack.caller.is_none());
		assert!(calling_stack().is_none());

		CALLING_STACK
			.scope(Some(current_stack.clone()), async move {
				assert!(calling_stack().is_some());

				let tracker = Arc::new(Tracker::new(current_stack));
				let result = tracker
					.track::<_, _>(async {
						tokio::time::sleep(Duration::from_secs(40)).await;
						Ok(())
					})
					.await;
				assert!(result.is_err());
				let error = result.unwrap_err();
				assert!(matches!(error, Global::InvocationTimeout(_)));
			})
			.await;
	}

	#[tokio::test]
	#[ignore]
	async fn hug_trackers_works() {
		let test_actor: ActorId = b"test_actor".to_vec().into_actor();
		let current_stack = CallingStack::step(test_actor);

		for _ in 0..10000 {
			let current_stack = current_stack.clone();
			tokio::spawn(async move {
				CALLING_STACK
					.scope(Some(current_stack.clone()), async move {
						let tracker = Arc::new(Tracker::new(current_stack));

						let mut futures = FuturesUnordered::new();
						let times = 100000;
						for _ in 0..times {
							futures.push(tracker.track::<_, _>(async { Ok(()) }));
						}

						let mut actual_counts = 0;
						while let Some(result) = futures.next().await {
							assert!(result.is_ok());
							actual_counts += 1;
						}
						assert_eq!(actual_counts, times);
					})
					.await;
			})
			.await
			.unwrap();
		}
	}
}
