use std::{future::Future, mem::MaybeUninit, pin::Pin, task::Poll};

use crate::core::worker_codec::Operation;

use crate::{
	actor::{Actor, ActorTAIT},
	error::Result,
};

use super::context::context;

#[doc(hidden)]
pub fn wasm_actor_entry<A>(op: Operation) -> Operation
where
	A: Actor + Default,
{
	match op {
		Operation::Call { ctx, req } => {
			match bincode::deserialize(&ctx) {
				Ok(c) => context().calling_stack = c,
				Err(e) => return Operation::ReturnErr { error: e.into() },
			}
			#[allow(clippy::uninit_assumed_init)]
			let mut execution = Box::pin(Interruptable(A::default(), req, unsafe {
				MaybeUninit::uninit().assume_init()
			}));
			unsafe {
				let execution = execution.as_mut().get_unchecked_mut();
				(&mut execution.2 as *mut <A as ActorTAIT>::Invoke<'static>).write(
					std::mem::transmute(ActorTAIT::invoke(&execution.0, &execution.1)),
				)
			}
			context().execution = Some(execution);
		}
		Operation::ReturnOk { resp } => context().input = Some(Ok(resp)),
		Operation::ReturnErr { error } => context().input = Some(Err(error)),
	}

	let execution = unsafe {
		Pin::new_unchecked(
			context()
				.execution
				.as_mut()
				.expect("Actor runtime internal error: there is not a current execution")
				.as_mut()
				.get_unchecked_mut()
				.downcast_mut_unchecked::<Interruptable<A>>(),
		)
	};

	let result = futures::executor::block_on(execution);
	if matches!(
		result,
		Operation::ReturnOk { .. } | Operation::ReturnErr { .. }
	) {
		context().calling_stack = None;
		context().execution = None;
	}
	result
}

pub(crate) struct Interrupt(pub Option<(Vec<u8>, Vec<u8>)>);

impl Future for Interrupt {
	type Output = Result<Vec<u8>>;
	fn poll(self: Pin<&mut Self>, _: &mut std::task::Context) -> Poll<Self::Output> {
		if let Some(input) = context().input.take() {
			return Poll::Ready(input);
		}
		let op =
			unsafe { self.get_unchecked_mut().0.take() }.expect("Polling an completed complete");
		context().output = Some(op);
		Poll::Pending
	}
}

struct Interruptable<A>(A, Vec<u8>, <A as ActorTAIT>::Invoke<'static>)
where
	A: Actor;

impl<A> Future for Interruptable<A>
where
	A: Actor,
{
	type Output = Operation;

	#[inline(always)]
	fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
		Poll::Ready(
			match unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().2) }.poll(cx) {
				Poll::Ready(Ok(resp)) => Operation::ReturnOk { resp },
				Poll::Ready(Err(e)) => Operation::ReturnErr { error: e.into() },
				Poll::Pending => {
					let (ctx, req) = context().output.take().expect("Unexpected interruption");
					Operation::Call { ctx, req }
				}
			},
		)
	}
}
