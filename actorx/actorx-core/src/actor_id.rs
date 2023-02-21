use std::{
	borrow::Borrow,
	fmt::{Debug, Display},
	hash::Hash,
	ops::Deref,
	str::FromStr,
	sync::Arc,
};

use serde::{Deserialize, Serialize};
use tea_codec::OptionExt;

use crate::error::{Error, Result};

#[derive(Clone, Eq)]
pub enum RegId {
	Shared(Arc<[u8]>),
	Static(&'static [u8]),
}

impl RegId {
	pub fn inst<I>(self, i: I) -> ActorId
	where
		I: Into<InstanceId>,
	{
		ActorId {
			reg: self,
			inst: i.into(),
		}
	}
}

impl Serialize for RegId {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.to_vec().serialize(serializer)
	}
}

impl<'de> Deserialize<'de> for RegId {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		Ok(Vec::<u8>::deserialize(deserializer)?.into())
	}
}

impl PartialEq for RegId {
	fn eq(&self, other: &Self) -> bool {
		self.as_ref() == other.as_ref()
	}
}

impl PartialEq<[u8]> for RegId {
	fn eq(&self, other: &[u8]) -> bool {
		self.as_ref() == other
	}
}

impl PartialEq<&[u8]> for RegId {
	fn eq(&self, other: &&[u8]) -> bool {
		self.as_ref() == *other
	}
}

impl PartialEq<Vec<u8>> for RegId {
	fn eq(&self, other: &Vec<u8>) -> bool {
		self.as_ref() == other.as_slice()
	}
}

impl Hash for RegId {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.as_ref().hash(state);
	}
}

impl Debug for RegId {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		Debug::fmt(&String::from(self), f)
	}
}

impl Display for RegId {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		Display::fmt(&String::from(self), f)
	}
}

impl Deref for RegId {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		match self {
			RegId::Shared(id) => id,
			RegId::Static(id) => id,
		}
	}
}

impl Borrow<[u8]> for RegId {
	fn borrow(&self) -> &[u8] {
		self
	}
}

impl AsRef<[u8]> for RegId {
	fn as_ref(&self) -> &[u8] {
		self
	}
}

impl From<Vec<u8>> for RegId {
	fn from(value: Vec<u8>) -> Self {
		Self::Shared(Arc::from(value.into_boxed_slice()))
	}
}

impl From<&'static [u8]> for RegId {
	fn from(value: &'static [u8]) -> Self {
		Self::Static(value)
	}
}

impl Default for RegId {
	fn default() -> Self {
		Self::Static(&[])
	}
}

impl From<&RegId> for String {
	fn from(value: &RegId) -> Self {
		match std::str::from_utf8(value) {
			Ok(s) => s.to_owned(),
			Err(_) => {
				let mut result = base64::encode(value);
				result.insert(0, '#');
				result
			}
		}
	}
}

impl From<RegId> for String {
	fn from(value: RegId) -> Self {
		String::from(&value)
	}
}

impl FromStr for RegId {
	type Err = Error;
	fn from_str(s: &str) -> Result<Self> {
		Ok(s.into())
	}
}

impl From<&str> for RegId {
	fn from(s: &str) -> Self {
		s.strip_prefix('#')
			.and_then(|s| base64::decode(s).ok())
			.unwrap_or_else(|| s.to_owned().into_bytes())
			.into()
	}
}

impl From<&String> for RegId {
	fn from(value: &String) -> Self {
		value.as_str().into()
	}
}

impl From<String> for RegId {
	fn from(value: String) -> Self {
		value.as_str().into()
	}
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(transparent)]
pub struct InstanceId(u128);

impl InstanceId {
	pub const ZERO: InstanceId = InstanceId(0);
}

impl Debug for InstanceId {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		Debug::fmt(&self.0, f)
	}
}
impl Display for InstanceId {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		Display::fmt(&self.0, f)
	}
}

impl From<u128> for InstanceId {
	fn from(value: u128) -> Self {
		Self(value)
	}
}

impl From<InstanceId> for u128 {
	fn from(value: InstanceId) -> Self {
		value.0
	}
}

impl PartialEq<u128> for InstanceId {
	fn eq(&self, other: &u128) -> bool {
		&self.0 == other
	}
}

impl PartialEq<InstanceId> for u128 {
	fn eq(&self, other: &InstanceId) -> bool {
		self == &other.0
	}
}

impl FromStr for InstanceId {
	type Err = <u128 as FromStr>::Err;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self(s.parse()?))
	}
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct ActorId {
	pub reg: RegId,
	pub inst: InstanceId,
}

impl Debug for ActorId {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.write_fmt(format_args!("{}:{}", self.reg, self.inst))
	}
}

impl Display for ActorId {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		Debug::fmt(self, f)
	}
}

impl From<ActorId> for String {
	fn from(value: ActorId) -> Self {
		value.to_string()
	}
}

impl FromStr for ActorId {
	type Err = Error;
	fn from_str(s: &str) -> Result<Self> {
		let (sp, ..) = s.rmatch_indices(':').next().ok_or_err(":")?;
		let reg = s[..sp].into();
		let inst = s[sp + 1..].parse()?;
		Ok(Self { reg, inst })
	}
}
