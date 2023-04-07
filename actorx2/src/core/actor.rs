use std::{
	borrow::Borrow,
	fmt::{Debug, Display, Formatter, UpperHex},
	hash::{Hash, Hasher},
	ops::Deref,
	sync::Arc,
};

use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub enum ActorId {
	Shared(Arc<[u8]>),
	Static(&'static [u8]),
}

impl Deref for ActorId {
	type Target = [u8];
	fn deref(&self) -> &Self::Target {
		match self {
			ActorId::Shared(value) => value,
			ActorId::Static(value) => value,
		}
	}
}

impl AsRef<[u8]> for ActorId {
	fn as_ref(&self) -> &[u8] {
		self
	}
}

impl Borrow<[u8]> for ActorId {
	fn borrow(&self) -> &[u8] {
		self
	}
}

impl PartialEq for ActorId {
	fn eq(&self, other: &Self) -> bool {
		**self == **other
	}
}

impl PartialEq<[u8]> for ActorId {
	fn eq(&self, other: &[u8]) -> bool {
		**self == *other
	}
}

impl PartialEq<ActorId> for [u8] {
	fn eq(&self, other: &ActorId) -> bool {
		*self == **other
	}
}

impl PartialEq<str> for ActorId {
	fn eq(&self, other: &str) -> bool {
		**self == *other.as_bytes()
	}
}

impl PartialEq<ActorId> for str {
	fn eq(&self, other: &ActorId) -> bool {
		*self.as_bytes() == **other
	}
}

impl Eq for ActorId {}

impl Hash for ActorId {
	fn hash<H: ~const Hasher>(&self, state: &mut H) {
		(**self).hash(state)
	}
}

impl Debug for ActorId {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		Display::fmt(self, f)
	}
}

impl Display for ActorId {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		if let Ok(s) = std::str::from_utf8(self) {
			Display::fmt(s, f)?;
			return Ok(());
		}

		Display::fmt(&'#', f)?;
		for n in &**self {
			if *n < 16 {
				UpperHex::fmt(&0, f)?;
			}
			UpperHex::fmt(&n, f)?;
		}
		Ok(())
	}
}

struct NotActorIdWrapper<T>(T);
auto trait NotActorId {}
impl !NotActorId for NotActorIdWrapper<ActorId> {}

impl<T> From<T> for ActorId
where
	T: AsRef<[u8]>,
	NotActorIdWrapper<T>: NotActorId,
{
	fn from(value: T) -> Self {
		Self::Shared(value.as_ref().into())
	}
}

impl From<ActorId> for Vec<u8> {
	fn from(value: ActorId) -> Self {
		value.to_vec()
	}
}

impl Serialize for ActorId {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.as_ref().serialize(serializer)
	}
}

impl<'de> Deserialize<'de> for ActorId {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		<&[u8] as Deserialize<'de>>::deserialize(deserializer).map(|x| Self::Shared(x.into()))
	}
}
