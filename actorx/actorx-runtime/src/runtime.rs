use std::{
	cell::RefCell,
	future::Future,
	pin::Pin,
	task::{Context, Poll},
};

use crate::{
	error::{Error, Result},
	interface::allocate_quote_id,
};
use tea_actorx_core::{
	actor::{decode_input, encode_invoke, OutputMessageKind},
	ActorId, RegId,
};
use tea_codec::{
	errorx::Scope,
	pricing::PricedOrDefault,
	serde::{handle::Request, FromBytes, ToBytes},
	ResultExt,
};

#[cfg(feature = "checked")]
use tea_codec::errorx::DescriptableMark;

tokio::task_local! {
	static ACTOR_INVOKE: RefCell<Option<Vec<u8>>>;
	static ACTOR_RETURN: RefCell<Option<Vec<u8>>>;
}

pub(crate) struct UseActorInvoke<F>(Option<F>);

impl<F> UseActorInvoke<F>
where
	F: Future,
{
	pub async fn new(f: F) -> <Self as Future>::Output {
		Self(Some(f)).await
	}
}

impl<F> Future for UseActorInvoke<F>
where
	F: Future,
{
	type Output = Result<F::Output, (Vec<u8>, Self)>;

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let slf = unsafe { &mut self.get_unchecked_mut().0 };
		let fut = unsafe {
			Pin::new_unchecked(slf.as_mut().expect("polling consumed host invoke future"))
		};

		match fut.poll(cx) {
			Poll::Ready(r) => Poll::Ready(Ok(r)),
			Poll::Pending => {
				let host_invoke =
					ACTOR_INVOKE.with(|actor_invoke| actor_invoke.borrow_mut().take());
				if let Some(output) = host_invoke {
					let slf = Self(slf.take());
					Poll::Ready(Err((output, slf)))
				} else {
					Poll::Pending
				}
			}
		}
	}
}

pub(crate) async fn with_actor_invoke<F>(f: F) -> F::Output
where
	F: Future,
{
	ACTOR_INVOKE.scope(RefCell::new(None), f).await
}

pub(crate) async fn with_actor_return<F>(msg: Vec<u8>, f: F) -> F::Output
where
	F: Future,
{
	ACTOR_RETURN.scope(RefCell::new(Some(msg)), f).await
}

fn invoke(
	reg_id: &[u8],
	actor_id: u128,
	msg: &impl ToBytes,
	budget: u64,
	waits: bool,
) -> Result<InvokeActor> {
	let quote_id = allocate_quote_id();
	let msg = encode_invoke(
		if waits {
			OutputMessageKind::HostCall
		} else {
			OutputMessageKind::HostPost
		},
		Some(quote_id),
		&RegId::from(reg_id.to_vec()).inst(actor_id),
		budget,
		msg,
	)?;

	Ok(InvokeActor { msg: Some(msg) })
}

struct InvokeActor {
	msg: Option<Vec<u8>>,
}

impl Future for InvokeActor {
	type Output = Vec<u8>;
	fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
		if let Some(msg) = self.msg.take() {
			ACTOR_INVOKE.with(|actor_invoke| *actor_invoke.borrow_mut() = Some(msg));
			return Poll::Pending;
		}

		if let Ok(Some(msg)) =
			ACTOR_RETURN.try_with(|actor_return| actor_return.borrow_mut().take())
		{
			return Poll::Ready(msg);
		}

		Poll::Pending
	}
}

#[cfg(feature = "checked")]
pub async fn cast<C, S>(actor_id: impl Into<ActorId>, arg: C) -> Result<C::Response, Error<S>>
where
	C: Request + ToBytes,
	C::Response: for<'a> FromBytes<'a>,
	S: Scope + DescriptableMark<Box<bincode::ErrorKind>>,
{
	let actor_id = actor_id.into();
	let input = arg.to_bytes()?;
	let result = invoke(&actor_id.reg, actor_id.inst.into(), &input).await;
	match result.kind() {
		tea_actorx_core::actor::InputMessageKind::HostReturn => {
			C::Response::from_bytes(&result).err_into()
		}
		tea_actorx_core::actor::InputMessageKind::HostError => Err(deserialize(&result)?),
		_ => panic!(),
	}
}

#[cfg(not(feature = "checked"))]
pub async fn call<C, S>(actor_id: impl Into<ActorId>, arg: C) -> Result<C::Response, Error<S>>
where
	C: Request + ToBytes,
	C::Response: for<'a> FromBytes<'a>,
	S: Scope,
{
	let actor_id = actor_id.into();
	let msg = invoke(
		&actor_id.reg,
		actor_id.inst.into(),
		&arg,
		arg.price().unwrap_or(0),
		true,
	)?
	.await;
	let (kind, _, _, msg) = decode_input(&msg)?;
	match kind {
		tea_actorx_core::actor::InputMessageKind::HostReturn => {
			C::Response::from_bytes(msg).err_into()
		}
		tea_actorx_core::actor::InputMessageKind::HostError => Err(Error::from_bytes(msg)?),
		_ => panic!(),
	}
}

pub async fn post<C, S>(actor_id: impl Into<ActorId>, arg: C) -> Result<(), Error<S>>
where
	C: Request<Response = ()> + ToBytes,
	S: Scope,
{
	let actor_id = actor_id.into();
	let msg = invoke(
		&actor_id.reg,
		actor_id.inst.into(),
		&arg,
		arg.price().unwrap_or(0),
		false,
	)?
	.await;
	let (kind, _, _, msg) = decode_input(&msg)?;
	match kind {
		tea_actorx_core::actor::InputMessageKind::HostReturn => Ok(()),
		tea_actorx_core::actor::InputMessageKind::HostError => Err(Error::from_bytes(msg)?),
		_ => panic!(),
	}
}

pub async fn post_with_budget<C, S>(
	actor_id: impl Into<ActorId>,
	arg: C,
	budget: u64,
) -> Result<(), Error<S>>
where
	C: Request<Response = ()> + ToBytes,
	S: Scope,
{
	let actor_id = actor_id.into();
	let msg = invoke(
		&actor_id.reg,
		actor_id.inst.into(),
		&arg,
		arg.price().unwrap_or(0).max(budget),
		false,
	)?
	.await;
	let (kind, _, _, msg) = decode_input(&msg)?;
	match kind {
		tea_actorx_core::actor::InputMessageKind::HostReturn => Ok(()),
		tea_actorx_core::actor::InputMessageKind::HostError => Err(Error::from_bytes(msg)?),
		_ => panic!(),
	}
}
