use std::{any::TypeId, borrow::Cow};

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use super::{DescriptableMark, Descriptor, Error, Global};

#[derive(Serialize, Deserialize)]
pub(crate) struct SerializedData<'a> {
	name: Cow<'a, str>,
	summary: Option<Cow<'a, str>>,
	detail: Option<Cow<'a, str>>,
	backtrace: Option<Cow<'a, str>>,
	pub(crate) inner: Option<SmallVec<[Cow<'a, Error>; 1]>>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SerializedDataWithHuman<'a> {
	name: Cow<'a, str>,
	summary: Option<Cow<'a, str>>,
	detail: Option<Cow<'a, str>>,
	human: Option<String>,
	backtrace: Option<Cow<'a, str>>,
	pub(crate) inner: Option<SmallVec<[Cow<'a, Error>; 1]>>,
}

impl SerializedData<'_> {
	fn into_owned(self) -> SerializedData<'static> {
		SerializedData {
			name: self.name.into_owned().into(),
			summary: self.summary.map(|x| x.into_owned().into()),
			detail: self.detail.map(|x| x.into_owned().into()),
			backtrace: self.backtrace.map(|x| x.into_owned().into()),
			inner: self
				.inner
				.map(|x| x.into_iter().map(|x| Cow::Owned(x.into_owned())).collect()),
		}
	}
}

impl<'a> DescriptableMark<SerializedData<'a>> for Global {}

impl<'a> Descriptor<SerializedData<'a>> for Global {
	fn name<'x>(v: &'x SerializedData<'a>) -> Option<Cow<'x, str>> {
		Some(Cow::Borrowed(&v.name))
	}

	fn summary<'x>(v: &'x SerializedData<'a>) -> Option<Cow<'x, str>> {
		v.summary.as_deref().map(Cow::Borrowed)
	}

	fn detail<'x>(v: &'x SerializedData<'a>) -> Option<Cow<'x, str>> {
		v.detail.as_deref().map(Cow::Borrowed)
	}

	fn inner<'x>(v: &'x SerializedData<'a>) -> Option<smallvec::SmallVec<[&'x Error; 1]>> {
		v.inner
			.as_ref()
			.map(|x| x.iter().map(|x| x.as_ref()).collect())
	}

	fn type_id(_: &SerializedData) -> Option<std::any::TypeId> {
		Some(TypeId::of::<SerializedData>())
	}
}

impl Serialize for Error {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		SerializeSpec::serialize(self, serializer)
	}
}

trait SerializeSpec<S>
where
	S: serde::Serializer,
{
	fn serialize(&self, serializer: S) -> Result<S::Ok, S::Error>;
}

impl<S> SerializeSpec<S> for Error
where
	S: serde::Serializer,
{
	default fn serialize(&self, serializer: S) -> Result<S::Ok, S::Error> {
		self.to_serialized_data().serialize(serializer)
	}
}

impl<'a, W, F> SerializeSpec<&'a mut serde_json::Serializer<W, F>> for Error
where
	&'a mut serde_json::Serializer<W, F>: serde::Serializer,
{
	fn serialize(
		&self,
		serializer: &'a mut serde_json::Serializer<W, F>,
	) -> Result<
		<&'a mut serde_json::Serializer<W, F> as serde::Serializer>::Ok,
		<&'a mut serde_json::Serializer<W, F> as serde::Serializer>::Error,
	> {
		self.to_serialized_data_with_human().serialize(serializer)
	}
}

impl<'a> Deserialize<'a> for Error {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'a>,
	{
		DeserializeSpec::deserialize(deserializer)
	}
}

trait DeserializeSpec<'a, D>: Sized
where
	D: serde::Deserializer<'a>,
{
	fn deserialize(deserializer: D) -> Result<Self, D::Error>;
}

impl<'a, D> DeserializeSpec<'a, D> for Error
where
	D: serde::Deserializer<'a>,
{
	default fn deserialize(deserializer: D) -> Result<Self, D::Error> {
		SerializedData::deserialize(deserializer)
			.map(|data| {
				#[cfg(feature = "backtrace")]
				let mut data = data;
				#[cfg(feature = "backtrace")]
				let backtrace = data.backtrace.take().unwrap();
				Error::new::<_, Global>(
					data,
					#[cfg(feature = "backtrace")]
					super::ErrorLocation::Serialized(backtrace),
				)
			})
			.map(Into::into)
	}
}

impl<'a, 'b, R> DeserializeSpec<'a, &'b mut serde_json::Deserializer<R>> for Error
where
	&'b mut serde_json::Deserializer<R>: serde::Deserializer<'a>,
{
	fn deserialize(
		deserializer: &'b mut serde_json::Deserializer<R>,
	) -> Result<Self, <&'b mut serde_json::Deserializer<R> as serde::Deserializer<'a>>::Error> {
		SerializedDataWithHuman::deserialize(deserializer)
			.map(|data| {
				#[cfg(feature = "backtrace")]
				let mut data = data;
				#[cfg(feature = "backtrace")]
				let backtrace = data.backtrace.take().unwrap();
				Error::new(
					SerializedData {
						name: data.name,
						summary: data.summary,
						detail: data.detail,
						backtrace: data.backtrace,
						inner: data.inner,
					},
					#[cfg(feature = "backtrace")]
					super::ErrorLocation::Serialized(backtrace),
				)
			})
			.map(Into::into)
	}
}

impl Clone for Error {
	fn clone(&self) -> Self {
		let result: Error = self.to_serialized_data().into_owned().into();
		result.into_scope()
	}
}

impl Error {
	fn to_serialized_data(&self) -> SerializedData {
		SerializedData {
			name: self.name(),
			summary: self.summary(),
			detail: self.detail(),
			#[cfg(feature = "backtrace")]
			backtrace: Some(self.backtrace()),
			#[cfg(not(feature = "backtrace"))]
			backtrace: None,
			inner: self
				.data
				.source
				.inner()
				.map(|inner| inner.into_iter().map(Cow::Borrowed).collect()),
		}
	}

	fn to_serialized_data_with_human(&self) -> SerializedDataWithHuman {
		SerializedDataWithHuman {
			name: self.name(),
			summary: self.summary(),
			detail: self.detail(),
			human: self.human(),
			#[cfg(feature = "backtrace")]
			backtrace: Some(self.backtrace()),
			#[cfg(not(feature = "backtrace"))]
			backtrace: None,
			inner: self
				.data
				.source
				.inner()
				.map(|inner| inner.into_iter().map(Cow::Borrowed).collect()),
		}
	}
}
