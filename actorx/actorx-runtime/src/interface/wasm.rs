use std::{cell::UnsafeCell, collections::HashMap, future::Future, pin::Pin};

use tea_actorx_core::{
	actor::{decode_input, encode_output, InputMessageKind, OutputMessageKind},
	hook::{PostInvoke, PreInvoke},
};
use tea_codec::{
	errorx::Global,
	serde::{
		handle::{Handle, Handles},
		layout::UseFromToBytes,
		ToBytes,
	},
};

use crate::{error::Error, with_actor_invoke, with_actor_return, CallingCx};

type UseActorInvoke<F = Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>>>>> =
	crate::runtime::UseActorInvoke<F>;

struct Context(UnsafeCell<ContextState>);
unsafe impl Send for Context {}
unsafe impl Sync for Context {}

struct ContextState {
	last_invoke: Option<HashMap<usize, UseActorInvoke>>,
	current: usize,
}

static CONTEXT: Context = Context(UnsafeCell::new(ContextState {
	last_invoke: None,
	current: 0,
}));

#[doc(hidden)]
pub async fn handle<A>(input: Vec<u8>) -> Vec<u8>
where
	A: Handles<CallingCx> + Default + 'static,
{
	let (kind, quote_id, ..) = decode_input(&input).expect("Failed to decode input");
	let context = unsafe { &mut *CONTEXT.0.get() };
	if context.last_invoke.is_none() {
		context.last_invoke = Some(HashMap::new());
	}
	let invoke = match kind {
		InputMessageKind::GuestCall => {
			with_actor_invoke(UseActorInvoke::new(Box::pin(async move {
				let (.., caller, msg) = decode_input(&input)?;
				if let Err(e) = A::default()
					.handle(&PreInvoke.to_bytes()?, caller.clone())
					.await
				{
					if e.name() != Global::UnexpectedType {
						return Err(e.into());
					}
				}
				let result = A::default().handle(msg, caller.clone()).await?;
				if let Err(e) = A::default().handle(&PostInvoke.to_bytes()?, caller).await {
					if e.name() != Global::UnexpectedType {
						return Err(e.into());
					}
				}
				Ok(result)
			})
				as Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>>>>))
			.await
		}
		InputMessageKind::HostReturn | InputMessageKind::HostError => {
			let last = context
				.last_invoke
				.as_mut()
				.unwrap()
				.remove(&quote_id.expect("No quote id provided."))
				.expect("HostReturn but there the quote is not found.");
			with_actor_invoke(with_actor_return(input, last)).await
		}
	};
	match invoke {
		Ok(Ok(payload)) => encode_output::<Vec<u8>>(OutputMessageKind::GuestReturn, None, &payload)
			.expect("Failed to encode return message"),
		Ok(Err(e)) => {
			encode_output::<UseFromToBytes<Error>>(OutputMessageKind::GuestError, None, &e)
				.expect("Failed to encode return error message")
		}
		Err((msg, invoke)) => {
			context
				.last_invoke
				.as_mut()
				.unwrap()
				.insert(context.current, invoke);
			msg
		}
	}
}

pub(crate) fn allocate_quote_id() -> usize {
	let context = unsafe { &mut *CONTEXT.0.get() };
	loop {
		if context
			.last_invoke
			.as_ref()
			.unwrap()
			.get(&context.current)
			.is_none()
		{
			return context.current;
		}
		context.current += 1;
		if context.current == usize::MAX {
			context.current = 0;
		}
	}
}
