#[cfg(feature = "host")]
use ::{
	std::{future::Future, sync::Arc},
	tea_sdk::errorx::Scope,
	tokio::task_local,
};

#[cfg(feature = "host")]
use crate::{context::tracker::Tracker, error::Error};

use crate::{
	core::actor::ActorId,
	error::{MissingCallingStack, Result},
	CallingStack,
};

#[cfg(feature = "host")]
task_local! {
	static CALLING_STACK: Option<CallingStack>;
	static TRACKER: Arc<Tracker>;
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
}
