#[cfg(feature = "host")]
use std::future::Future;
use std::{ops::Deref, sync::Arc};

use crate::core::actor::ActorId;
use serde::{
	de::{SeqAccess, Visitor},
	ser::SerializeSeq,
	Deserialize, Serialize, Serializer,
};
#[cfg(feature = "host")]
use tokio::task_local;

use crate::error::{MissingCallingStack, Result};

#[cfg(feature = "host")]
task_local! {
	static CALLING_STACK: Option<CallingStack>;
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
	async fn with_calling_stack(self, value: Option<CallingStack>) -> Self::Output;
	async fn invoke_target(self, value: ActorId) -> Self::Output;
}

#[cfg(feature = "host")]
impl<T> WithCallingStack for T
where
	T: Future,
{
	async fn with_calling_stack(self, value: Option<CallingStack>) -> Self::Output {
		CALLING_STACK.scope(value, self).await
	}

	async fn invoke_target(self, value: ActorId) -> Self::Output {
		CALLING_STACK
			.scope(Some(CallingStack::step(value)), self)
			.await
	}
}

#[derive(Clone, Debug)]
pub struct CallingStack(Arc<CallingNode>);

impl CallingStack {
	pub fn step(current: ActorId) -> Self {
		Self(Arc::new(CallingNode {
			current,
			caller: calling_stack(),
		}))
	}
}

impl Deref for CallingStack {
	type Target = CallingNode;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Clone, Debug)]
pub struct CallingNode {
	pub current: ActorId,
	pub caller: Option<CallingStack>,
}

#[derive(Clone, Debug)]
pub struct Iter(Option<CallingStack>);

impl Iterator for Iter {
	type Item = ActorId;
	fn next(&mut self) -> Option<Self::Item> {
		self.0.take().map(|v| {
			self.0 = v.caller.clone();
			v.current.clone()
		})
	}
}

impl IntoIterator for CallingStack {
	type Item = ActorId;
	type IntoIter = Iter;
	fn into_iter(self) -> Self::IntoIter {
		Iter(Some(self))
	}
}

impl Serialize for CallingStack {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		fn rev_stack<S>(
			serializer: S,
			current: &CallingStack,
			count: usize,
		) -> Result<<S as serde::Serializer>::SerializeSeq, S::Error>
		where
			S: serde::Serializer,
		{
			let count = count + 1;
			let mut seq = if let Some(caller) = &current.caller {
				rev_stack::<S>(serializer, caller, count)?
			} else {
				serializer.serialize_seq(Some(count))?
			};
			seq.serialize_element(current.current.as_ref())?;
			Ok(seq)
		}
		rev_stack::<S>(serializer, self, 0)?.end()
	}
}

impl<'de> Deserialize<'de> for CallingStack {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct CallingStackVisitor;

		impl<'de> Visitor<'de> for CallingStackVisitor {
			type Value = CallingStack;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("calling stack")
			}

			fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
			where
				A: SeqAccess<'de>,
			{
				let mut result = None;
				while let Some(id) = seq.next_element()? as Option<&[u8]> {
					let current = ActorId::Shared(id.into());
					let stack = CallingStack(Arc::new(CallingNode {
						current,
						caller: result,
					}));
					result = Some(stack);
				}
				result.ok_or_else(|| serde::de::Error::invalid_length(0, &"at least 1 stack node"))
			}
		}

		deserializer.deserialize_seq(CallingStackVisitor)
	}
}
