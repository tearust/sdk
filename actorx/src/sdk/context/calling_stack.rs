#[cfg(feature = "host")]
#[cfg(feature = "timeout")]
use std::sync::Arc;
#[cfg(feature = "host")]
use ::{std::future::Future, tea_sdk::errorx::Scope, tokio::task_local};

#[cfg(feature = "host")]
use crate::error::Error;

#[cfg(feature = "host")]
#[cfg(feature = "timeout")]
use crate::context::tracker::Tracker;

use crate::{
	core::actor::ActorId,
	error::{MissingCallingStack, Result},
	CallingStack,
};

#[cfg(feature = "host")]
task_local! {
	pub(crate) static CALLING_STACK: Option<CallingStack>;
	#[cfg(feature = "timeout")]
	pub(crate) static TRACKER: Arc<Tracker>;
}

#[inline(always)]
pub fn calling_stack() -> Option<CallingStack> {
	#[cfg(feature = "host")]
	return CALLING_STACK.try_with(|x| x.clone()).ok().flatten();
	#[cfg(not(feature = "host"))]
	return crate::wasm::context::context().calling_stack.clone();
}

#[inline(always)]
pub fn current() -> Result<ActorId> {
	current_ref(|x| x.clone())
}

#[inline(always)]
pub(crate) fn current_ref<O>(f: impl FnOnce(&ActorId) -> O) -> Result<O> {
	#[cfg(feature = "host")]
	return CALLING_STACK
		.try_with(|x| x.as_ref().map(|x| f(&x.current)))
		.ok()
		.flatten()
		.ok_or_else(|| MissingCallingStack::Current.into());
	#[cfg(not(feature = "host"))]
	return crate::wasm::context::context()
		.calling_stack
		.as_ref()
		.map(|x| f(&x.current))
		.ok_or_else(|| MissingCallingStack::Current.into());
}

#[inline(always)]
pub fn caller() -> Result<Option<ActorId>> {
	#[cfg(feature = "host")]
	return CALLING_STACK
		.try_with(|x| {
			x.as_ref()
				.map(|x| x.caller.as_ref().map(|x| x.current.clone()))
		})
		.ok()
		.flatten()
		.ok_or_else(|| MissingCallingStack::Caller.into());
	#[cfg(not(feature = "host"))]
	return Ok(crate::wasm::context::context()
		.calling_stack
		.as_ref()
		.map(|x| x.current.clone()));
}

#[cfg(feature = "host")]
pub(crate) trait WithCallingStack: Future {
	async fn invoke_target(self, value: ActorId) -> Self::Output;
}

#[cfg(all(feature = "host", feature = "track"))]
#[inline(always)]
pub(crate) fn full_stack() -> Option<Arc<tokio::sync::RwLock<CallingStack>>> {
	TRACKER
		.try_with(|tracker| tracker.full_stack().clone())
		.ok()
}

#[cfg(feature = "host")]
impl<T, R, S> WithCallingStack for T
where
	T: Future<Output = Result<R, Error<S>>>,
	S: Scope,
{
	#[inline(always)]
	async fn invoke_target(self, value: ActorId) -> Self::Output {
		let stack = CallingStack::step(value);
		#[cfg(feature = "timeout")]
		{
			let is_first = stack.0.caller.is_none();
			let f = async move {
				let tracker = TRACKER.with(Arc::clone);
				tracker.track(self).await
			};
			#[cfg(feature = "track")]
			let fut = CALLING_STACK.scope(Some(stack.clone()), f);
			#[cfg(not(feature = "track"))]
			let fut = CALLING_STACK.scope(Some(stack), f);
			if is_first {
				TRACKER
					.scope(
						Arc::new(Tracker::new(
							#[cfg(feature = "track")]
							stack,
						)),
						fut,
					)
					.await
			} else {
				#[cfg(feature = "track")]
				{
					*TRACKER
						.with(|tracker| tracker.full_stack().clone())
						.write_owned()
						.await = stack;
				}
				fut.await
			}
		}
		#[cfg(not(feature = "timeout"))]
		CALLING_STACK.scope(Some(stack), self).await
	}
}

#[cfg(test)]
mod tests {
	use futures::future::join_all;

	use crate::context::WithCallingStack;
	use crate::error::Result;
	use crate::{calling_stack, current, ActorId};

	const TEST_ACTOR: &[u8] = b"test_actor";
	const TEST_ACTOR2: &[u8] = b"test_actor2";

	#[tokio::test]
	async fn single_actor_invoke_target_works() {
		async fn fun1(req: &[u8]) -> Result<usize> {
			assert!(calling_stack().is_some());
			let stack = calling_stack().unwrap();
			println!("fun1 calling stack: {:?}", stack);
			assert_eq!(stack.current, ActorId::Static(TEST_ACTOR));
			assert!(stack.caller.is_none());
			Ok(req.len())
		}

		async fn fun2_1(req: &[u8]) -> Result<usize> {
			assert!(calling_stack().is_some());
			let root = calling_stack().unwrap();
			println!("fun2_1 calling stack: {:?}", root);
			assert_eq!(root.current, ActorId::Static(TEST_ACTOR));
			// nested caller
			assert!(root.caller.is_some());

			let child = root.caller.as_ref().unwrap();
			assert_eq!(child.current, ActorId::Static(TEST_ACTOR));
			assert!(child.caller.is_none());
			Ok(req.len())
		}

		async fn fun2_2(req: &[u8]) -> Result<usize> {
			assert!(calling_stack().is_some());
			let stack = calling_stack().unwrap();
			println!("fun2_2 calling stack: {:?}", stack);
			assert_eq!(stack.current, ActorId::Static(TEST_ACTOR));
			// caller is none
			assert!(stack.caller.is_none());
			fun2_1(req).invoke_target(current()?).await.map(|x| x + 1)
		}

		let test_actor = ActorId::Static(TEST_ACTOR);
		let test_request = b"test_request";

		let result = fun1(test_request)
			.invoke_target(test_actor.clone())
			.await
			.unwrap();
		assert_eq!(result, test_request.len());

		let result2 = fun2_2(test_request)
			.invoke_target(test_actor.clone())
			.await
			.unwrap();
		assert_eq!(result2, test_request.len() + 1);
	}

	#[tokio::test]
	async fn multi_actor_invoke_target_works() {
		async fn fun1(req: &[u8]) -> Result<usize> {
			assert!(calling_stack().is_some());
			let root = calling_stack().unwrap();
			println!("fun1 calling stack: {:?}", root);
			assert_eq!(root.current, ActorId::Static(TEST_ACTOR2));
			// nested caller
			assert!(root.caller.is_some());

			let child = root.caller.as_ref().unwrap();
			assert_eq!(child.current, ActorId::Static(TEST_ACTOR));
			assert!(child.caller.is_none());
			Ok(req.len())
		}

		async fn fun2(req: &[u8]) -> Result<usize> {
			assert!(calling_stack().is_some());
			let stack = calling_stack().unwrap();
			println!("fun2 calling stack: {:?}", stack);
			assert_eq!(stack.current, ActorId::Static(TEST_ACTOR));
			// caller is none
			assert!(stack.caller.is_none());

			let actor2 = ActorId::Static(TEST_ACTOR2);
			fun1(req).invoke_target(actor2).await.map(|x| x + 1)
		}

		let test_actor = ActorId::Static(TEST_ACTOR);
		let test_request = b"test_request";

		let result = fun2(test_request)
			.invoke_target(test_actor.clone())
			.await
			.unwrap();
		assert_eq!(result, test_request.len() + 1);
	}

	#[tokio::test]
	#[ignore]
	async fn hug_invoke_target_works() {
		async fn fun1(req: &[u8]) -> Result<usize> {
			// println!("fun1 calling stack: {:?}", calling_stack().unwrap());
			Ok(req.len())
		}

		async fn fun2(req: &[u8]) -> Result<usize> {
			let actor2 = ActorId::Static(TEST_ACTOR2);
			fun1(req).invoke_target(actor2).await.map(|x| x + 1)
		}

		let join_handles = (0..1000000).map(|_| {
			let test_actor = ActorId::Static(TEST_ACTOR);
			let test_request = b"test_request";

			tokio::spawn(async move {
				let result = fun2(test_request)
					.invoke_target(test_actor.clone())
					.await
					.unwrap();
				assert_eq!(result, test_request.len() + 1);
			})
		});

		join_all(join_handles).await;
	}
}
