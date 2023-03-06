use std::{
	any::type_name,
	fmt::Debug,
	future::Future,
	ops::{Deref, DerefMut},
	pin::Pin,
	sync::Arc,
};

use smallvec::{smallvec, SmallVec};
use tea_actorx_core::ActorId;
use tokio::task_local;

use crate::{error::Result, ActorHostRef};

pub(crate) mod looped;
mod native;
pub(crate) mod shared;
mod wasm;
pub use native::*;
pub use wasm::WasmActorFactory;

use self::{
	looped::{Actor, Looped},
	shared::Shared,
};

pub type DynFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Clone)]
pub struct ActorContext {
	pub host: ActorHostRef,
	pub id: ActorId,
}

#[derive(Debug, Clone)]
pub struct CallingContext {
	pub actor: ActorContext,
	pub caller: Option<ActorId>,
}

task_local! {
	static CALLING_CONTEXT: CallingContext;
}

pub async fn with_calling_context<O>(
	context: CallingContext,
	future: impl Future<Output = O>,
) -> O {
	CALLING_CONTEXT.scope(context, future).await
}

pub fn calling_context() -> Option<CallingContext> {
	CALLING_CONTEXT.try_with(|x| x.clone()).ok()
}

impl Deref for CallingContext {
	type Target = ActorContext;
	fn deref(&self) -> &Self::Target {
		&self.actor
	}
}

impl DerefMut for CallingContext {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.actor
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ActorKind {
	Native,
	Wasm,
}

#[derive(Clone)]
#[repr(transparent)]
pub struct ActorAgent(Arc<Impl>);

enum Impl {
	Looped(Looped),
	Shared(Shared),
	Multicast(Vec<ActorAgent>),
}

impl Debug for ActorAgent {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct(type_name::<Self>()).finish()
	}
}

impl ActorAgent {
	pub fn looped<T>(origin: T, context: ActorContext) -> Self
	where
		T: Actor,
	{
		Self(Arc::new(Impl::Looped(Looped::new(origin, context))))
	}

	pub fn shared<T>(origin: T, context: ActorContext) -> Self
	where
		T: shared::Actor,
	{
		Self(Arc::new(Impl::Shared(Shared::new(origin, context))))
	}

	pub fn multicast<T>(source: T) -> Self
	where
		T: Into<Vec<ActorAgent>>,
	{
		Self(Arc::new(Impl::Multicast(source.into())))
	}

	pub fn id(&self) -> &ActorId {
		match &*self.0 {
			Impl::Looped(looped) => looped.id(),
			Impl::Shared(shared) => shared.id(),
			Impl::Multicast(_) => unreachable!("Trying to get an id of a multicast actor agent"),
		}
	}

	pub fn kind(&self) -> ActorKind {
		match &*self.0 {
			Impl::Looped(looped) => looped.kind(),
			Impl::Shared(shared) => shared.kind(),
			Impl::Multicast(_) => unreachable!("Trying to get an id of a multicast actor agent"),
		}
	}
}

impl ActorAgent {
	pub async fn invoke(
		&self,
		msg: Vec<u8>,
		caller: Option<ActorId>,
	) -> Result<SmallVec<[Vec<u8>; 1]>> {
		Ok(match &self.0.as_ref() {
			Impl::Looped(looped) => {
				smallvec![looped.invoke(msg, caller).await?]
			}
			Impl::Shared(shared) => smallvec![shared.invoke(msg, caller).await?],
			Impl::Multicast(agents) => {
				let mut result = SmallVec::new();
				for agent in agents {
					result.extend(agent.invoke_dyn(msg.clone(), caller.clone()).await?);
				}
				result
			}
		})
	}

	fn invoke_dyn(
		&self,
		msg: Vec<u8>,
		caller: Option<ActorId>,
	) -> DynFuture<Result<SmallVec<[Vec<u8>; 1]>>> {
		Box::pin(self.invoke(msg, caller))
	}

	pub fn post(&self, msg: Vec<u8>, caller: Option<ActorId>) -> Result<()> {
		match &self.0.as_ref() {
			Impl::Looped(looped) => looped.post(msg, caller),
			Impl::Shared(shared) => shared.post(msg, caller),
			Impl::Multicast(agents) => {
				for agent in agents {
					agent.post(msg.clone(), caller.clone())?;
				}

				Ok(())
			}
		}
	}

	pub async fn activate(&self, caller: Option<ActorId>) -> Result<()> {
		match &self.0.as_ref() {
			Impl::Looped(looped) => looped.activate(caller).await,
			Impl::Shared(shared) => shared.activate(caller).await,
			Impl::Multicast(agents) => {
				for agent in agents {
					agent.activate_dyn(caller.clone()).await?;
				}
				Ok(())
			}
		}
	}

	fn activate_dyn(&self, caller: Option<ActorId>) -> DynFuture<Result<()>> {
		Box::pin(self.activate(caller))
	}
}
