//! The serde abstraction and serde based invocation dispatcher system.

use std::{io::Write, mem::size_of};

use bincode::Options;
use prost::bytes::BufMut;

use self::error::{Result, TypeIdMismatch};
use crate::{bincode_options, const_concat::ConstStr, errorx::BadBinaryFormat, ResultExt};
pub use tea_codec_macros::TypeId;

pub mod error;
pub mod handle;
pub mod layout;

#[rustc_specialization_trait]
trait Serialize = serde::Serialize;

#[rustc_specialization_trait]
trait Deserialize<'a> = serde::Deserialize<'a>;

#[rustc_specialization_trait]
trait Message = prost::Message;

#[rustc_specialization_trait]
trait MessageDefault = prost::Message + Default;

#[rustc_specialization_trait]
pub trait TypeId {
	const TYPE_ID: &'static str;
}

#[doc(hidden)]
pub trait TypeIdBuf {
	const TYPE_ID_BUF: ConstStr = ConstStr::empty();
}

pub trait ToBytes {
	fn to_bytes(&self) -> Result<Vec<u8>>;
	fn bytes_len(&self) -> Result<usize>;
	fn write_to(&self, w: impl SerBuf) -> Result<()>;
}

pub trait FromBytes<'a>: Sized {
	fn from_bytes(buf: &'a [u8]) -> Result<Self>;
}

#[rustc_specialization_trait]
trait ToBytesUsingSerialize {
	fn to_bytes(&self) -> Result<Vec<u8>>;
	fn bytes_len(&self) -> Result<usize>;
	fn write_to(&self, w: impl SerBuf) -> Result<()>;
}

#[rustc_specialization_trait]
trait FromBytesUsingSerialize<'a>: Sized {
	fn from_bytes(buf: &'a [u8]) -> Result<Self>;
}

pub trait SerBuf: Write + BufMut {
	fn reserve(&mut self, l: usize);
}

impl SerBuf for &mut Vec<u8> {
	fn reserve(&mut self, l: usize) {
		self.reserve_exact(l);
	}
}

impl SerBuf for &mut [u8] {
	fn reserve(&mut self, _: usize) {}
}

impl<T> SerBuf for &mut T
where
	T: SerBuf,
{
	fn reserve(&mut self, l: usize) {
		<T as SerBuf>::reserve(self, l)
	}
}

impl<T> ToBytesUsingSerialize for T
where
	T: TypeId + Serialize,
{
	fn to_bytes(&self) -> Result<Vec<u8>> {
		let mut result = Vec::new();
		ToBytesUsingSerialize::write_to(self, &mut result)?;
		Ok(result)
	}

	fn bytes_len(&self) -> Result<usize> {
		Ok(size_of::<u32>() + T::TYPE_ID.len() + bincode_options().serialized_size(self)? as usize)
	}

	fn write_to(&self, mut w: impl SerBuf) -> Result<()> {
		w.reserve(
			size_of::<u32>() + T::TYPE_ID.len() + bincode_options().serialized_size(self)? as usize,
		);

		w.write_all(&(T::TYPE_ID.len() as u32).to_le_bytes())?;

		w.write_all(T::TYPE_ID.as_bytes())?;

		bincode_options().serialize_into(w, self)?;

		Ok(())
	}
}

impl<'a, T> FromBytesUsingSerialize<'a> for T
where
	T: TypeId + Deserialize<'a>,
{
	fn from_bytes(buf: &'a [u8]) -> Result<Self> {
		if buf.len() < size_of::<u32>() {
			return Err(BadBinaryFormat.into());
		}
		let (type_id_len, buf) = buf.split_at(size_of::<u32>());
		let type_id_len =
			u32::from_le_bytes(unsafe { type_id_len.try_into().unwrap_unchecked() }) as _;
		if buf.len() < type_id_len {
			return Err(BadBinaryFormat.into());
		}
		let (type_id, payload) = buf.split_at(type_id_len);
		if type_id != T::TYPE_ID.as_bytes() {
			return Err(
				TypeIdMismatch(T::TYPE_ID, String::from_utf8_lossy(type_id).into_owned()).into(),
			);
		}

		bincode_options().deserialize(payload).err_into()
	}
}

pub fn get_type_id(buf: &[u8]) -> Result<&str> {
	if buf.len() < size_of::<u32>() {
		return Err(BadBinaryFormat.into());
	}
	let (type_id_len, buf) = buf.split_at(size_of::<u32>());
	let type_id_len = u32::from_le_bytes(unsafe { type_id_len.try_into().unwrap_unchecked() }) as _;
	if buf.len() < type_id_len {
		return Err(BadBinaryFormat.into());
	}
	std::str::from_utf8(&buf[..type_id_len]).err_into()
}

impl<T> ToBytesUsingSerialize for T
where
	T: Serialize,
{
	#[inline(always)]
	default fn to_bytes(&self) -> Result<Vec<u8>> {
		bincode_options().serialize(self).err_into()
	}

	#[inline(always)]
	default fn bytes_len(&self) -> Result<usize> {
		Ok(bincode_options().serialized_size(self)? as _)
	}

	#[inline(always)]
	default fn write_to(&self, w: impl SerBuf) -> Result<()> {
		bincode_options().serialize_into(w, self).err_into()
	}
}

impl<'a, T> FromBytesUsingSerialize<'a> for T
where
	T: Deserialize<'a>,
{
	#[inline(always)]
	default fn from_bytes(buf: &'a [u8]) -> Result<Self> {
		bincode_options().deserialize(buf).err_into()
	}
}

trait ToBytesUsingSerializeOrProto {
	fn to_bytes(&self) -> Result<Vec<u8>>;
	fn bytes_len(&self) -> Result<usize>;
	fn write_to(&self, w: impl SerBuf) -> Result<()>;
}

trait FromBytesUsingSerializeOrProto<'a>: Sized {
	fn from_bytes(buf: &'a [u8]) -> Result<Self>;
}

impl<T> ToBytesUsingSerializeOrProto for T {
	#[inline(always)]
	default fn to_bytes(&self) -> Result<Vec<u8>> {
		ToBytesUsingProtoOrPanic::to_bytes(self)
	}

	#[inline(always)]
	default fn bytes_len(&self) -> Result<usize> {
		ToBytesUsingProtoOrPanic::bytes_len(self)
	}

	#[inline(always)]
	default fn write_to(&self, w: impl SerBuf) -> Result<()> {
		ToBytesUsingProtoOrPanic::write_to(self, w)
	}
}

impl<'a, T> FromBytesUsingSerializeOrProto<'a> for T {
	#[inline(always)]
	default fn from_bytes(buf: &'a [u8]) -> Result<Self> {
		FromBytesUsingProtoOrPanic::from_bytes(buf)
	}
}

impl<T> ToBytesUsingSerializeOrProto for T
where
	T: ToBytesUsingSerialize,
{
	#[inline(always)]
	fn to_bytes(&self) -> Result<Vec<u8>> {
		ToBytesUsingSerialize::to_bytes(self)
	}

	#[inline(always)]
	fn bytes_len(&self) -> Result<usize> {
		ToBytesUsingSerialize::bytes_len(self)
	}

	#[inline(always)]
	fn write_to(&self, w: impl SerBuf) -> Result<()> {
		ToBytesUsingSerialize::write_to(self, w)
	}
}

impl<'a, T> FromBytesUsingSerializeOrProto<'a> for T
where
	T: FromBytesUsingSerialize<'a>,
{
	#[inline(always)]
	fn from_bytes(buf: &'a [u8]) -> Result<Self> {
		FromBytesUsingSerialize::from_bytes(buf)
	}
}

trait ToBytesUsingProtoOrPanic {
	fn to_bytes(&self) -> Result<Vec<u8>>;
	fn bytes_len(&self) -> Result<usize>;
	fn write_to(&self, w: impl SerBuf) -> Result<()>;
}

trait FromBytesUsingProtoOrPanic<'a>: Sized {
	fn from_bytes(buf: &'a [u8]) -> Result<Self>;
}

impl<T> ToBytesUsingProtoOrPanic for T {
	#[inline(always)]
	default fn to_bytes(&self) -> Result<Vec<u8>> {
		unimplemented!()
	}

	#[inline(always)]
	default fn bytes_len(&self) -> Result<usize> {
		unimplemented!()
	}

	#[inline(always)]
	default fn write_to(&self, _: impl SerBuf) -> Result<()> {
		unimplemented!()
	}
}

impl<'a, T> FromBytesUsingProtoOrPanic<'a> for T {
	#[inline(always)]
	default fn from_bytes(_: &'a [u8]) -> Result<Self> {
		unimplemented!()
	}
}

impl<T> ToBytesUsingProtoOrPanic for T
where
	T: Message,
{
	#[inline(always)]
	fn to_bytes(&self) -> Result<Vec<u8>> {
		Ok(self.encode_to_vec())
	}

	#[inline(always)]
	fn bytes_len(&self) -> Result<usize> {
		Ok(self.encoded_len())
	}

	#[inline(always)]
	fn write_to(&self, mut w: impl SerBuf) -> Result<()> {
		self.encode(&mut w).err_into()
	}
}

impl<'a, T> FromBytesUsingProtoOrPanic<'a> for T
where
	T: MessageDefault,
{
	#[inline(always)]
	fn from_bytes(buf: &'a [u8]) -> Result<Self> {
		T::decode(buf).err_into()
	}
}

#[marker]
trait IsToBytes {}

impl<T> IsToBytes for T where T: Serialize {}
impl<T> IsToBytes for T where T: Message {}
impl IsToBytes for () {}
impl IsToBytes for Vec<u8> {}

#[marker]
trait IsFromBytes<'a> {}

impl<'a, T> IsFromBytes<'a> for T where T: Deserialize<'a> {}
impl<'a, T> IsFromBytes<'a> for T where T: Message + Default {}
impl<'a> IsFromBytes<'a> for () {}
impl<'a> IsFromBytes<'a> for Vec<u8> {}

impl<T> ToBytes for T
where
	T: IsToBytes,
{
	#[inline(always)]
	fn to_bytes(&self) -> Result<Vec<u8>> {
		ToBytesUsingSerializeOrProto::to_bytes(self)
	}

	#[inline(always)]
	fn bytes_len(&self) -> Result<usize> {
		ToBytesUsingSerializeOrProto::bytes_len(self)
	}

	#[inline(always)]
	fn write_to(&self, w: impl SerBuf) -> Result<()> {
		ToBytesUsingSerializeOrProto::write_to(self, w)
	}
}

impl<'a, T> FromBytes<'a> for T
where
	T: IsFromBytes<'a>,
{
	#[inline(always)]
	default fn from_bytes(buf: &'a [u8]) -> Result<Self> {
		FromBytesUsingSerializeOrProto::from_bytes(buf)
	}
}

#[cfg(test)]
mod test {
	use serde::{Deserialize, Serialize};

	use crate::serde::{FromBytes, ToBytes};

	use super::TypeId;

	#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
	struct A(String);

	impl TypeId for A {
		const TYPE_ID: &'static str = "A";
	}

	#[test]
	fn test() {
		let source = A("TestValue".into());
		let bytes = source.to_bytes().unwrap();
		assert_ne!(
			format!("{bytes:?}"),
			"[9, 0, 0, 0, 0, 0, 0, 0, 84, 101, 115, 116, 86, 97, 108, 117, 101]"
		);
		let result = A::from_bytes(&bytes).unwrap();
		assert_eq!(source, result);
	}
}
