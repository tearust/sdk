use std::{
	fmt::{Debug, Display},
	iter::FusedIterator,
	ops::Deref,
	sync::Arc,
};

use serde::{
	de::{SeqAccess, Visitor},
	ser::SerializeSeq,
	Deserialize, Serialize, Serializer,
};

use crate::ActorId;

#[derive(Clone)]
pub struct CallingStack(pub(crate) Arc<CallingNode>);

#[cfg(any(feature = "host", feature = "wasm"))]
impl CallingStack {
	#[inline(always)]
	pub fn step(current: ActorId) -> Self {
		Self(Arc::new(CallingNode {
			current,
			caller: crate::calling_stack(),
		}))
	}
}

impl Deref for CallingStack {
	type Target = CallingNode;

	#[inline(always)]
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
pub struct IntoIter(Option<CallingStack>);

impl Iterator for IntoIter {
	type Item = ActorId;

	#[inline(always)]
	fn next(&mut self) -> Option<Self::Item> {
		self.0.take().map(|v| {
			self.0 = v.caller.clone();
			v.current.clone()
		})
	}
}

impl FusedIterator for IntoIter {}

#[derive(Clone, Debug)]
pub struct Iter<'a>(Option<&'a CallingStack>);

impl<'a> Iterator for Iter<'a> {
	type Item = &'a ActorId;

	#[inline(always)]
	fn next(&mut self) -> Option<Self::Item> {
		self.0.take().map(|v| {
			self.0 = v.caller.as_ref();
			&v.current
		})
	}
}

impl FusedIterator for Iter<'_> {}

impl IntoIterator for CallingStack {
	type Item = ActorId;
	type IntoIter = IntoIter;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		IntoIter(Some(self))
	}
}

impl<'a> IntoIterator for &'a CallingStack {
	type Item = &'a ActorId;
	type IntoIter = Iter<'a>;

	#[inline(always)]
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

impl Display for CallingStack {
	#[inline(always)]
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let mut list = f.debug_list();
		for actor in self {
			list.entry(actor);
		}
		list.finish()
	}
}

impl Debug for CallingStack {
	#[inline(always)]
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		Display::fmt(&self, f)
	}
}
