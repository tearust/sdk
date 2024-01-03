use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::serde::error::{InvalidFormat, TypeIdMismatch, UnexpectedType};

use super::{BadBinaryFormat, CannotBeNone, Global, RoutineTimeout};

#[derive(Debug, Serialize, Deserialize)]
pub enum GlobalErrorData {
	Unnamed(String),
	CannotBeNone(CannotBeNone),
	BadBinaryFormat(BadBinaryFormat),
	RoutineTimeout(RoutineTimeout),
	TypeIdMismatch(TypeIdMismatch),
	UnexpectedType(UnexpectedType),
	InvalidFormat(InvalidFormat),

	JsonSerde(String),
	BincodeSerde(String),
	Utf8Error(String),
	FromUtf8(String),
	StdIo(String),
	ProstDecode(String),
	ProstEncode(String),
	TryFromIntError(String),
	TryFromCharError(String),
	TryFromSliceError(String),
	ParseBool(String),
	ParseInt(String),
	ParseChar(String),
	ParseFloat(String),
	ParseAddr(String),
	SystemTime(String),
	ParseLog(String),
	SetLog(String),
	Base64Decode(String),
	HexDecode(String),
	MpscRecv(String),
	CrossbeamReceive(String),
	ChannelReceive(String),
	ChannelCanceled(String),
	ChannelSend(String),
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SerializedData<'a> {
	name: Cow<'a, str>,
	summary: Option<Cow<'a, str>>,
	detail: Option<Cow<'a, str>>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SerializedDataWithHuman<'a> {
	name: Cow<'a, str>,
	summary: Option<Cow<'a, str>>,
	detail: Option<Cow<'a, str>>,
	human: Option<String>,
}

impl Clone for Global {
	fn clone(&self) -> Self {
		let data = GlobalErrorData::from(self);
		data.into()
	}
}

impl PartialEq for Global {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Global::Unnamed(a), Global::Unnamed(b)) => a == b,
			(Global::CannotBeNone(a), Global::CannotBeNone(b)) => a == b,
			(Global::BadBinaryFormat(a), Global::BadBinaryFormat(b)) => a == b,
			(Global::RoutineTimeout(a), Global::RoutineTimeout(b)) => a == b,
			(Global::TypeIdMismatch(a), Global::TypeIdMismatch(b)) => a == b,
			(Global::UnexpectedType(a), Global::UnexpectedType(b)) => a == b,
			(Global::InvalidFormat(a), Global::InvalidFormat(b)) => a == b,

			(Global::JsonSerde(a), Global::JsonSerde(b)) => a.to_string() == b.to_string(),
			(Global::JsonSerde(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::JsonSerde(b)) => a == &b.to_string(),

			(Global::BincodeSerde(a), Global::BincodeSerde(b)) => a.to_string() == b.to_string(),
			(Global::BincodeSerde(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::BincodeSerde(b)) => a == &b.to_string(),

			(Global::Utf8Error(a), Global::Utf8Error(b)) => a.to_string() == b.to_string(),
			(Global::Utf8Error(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::Utf8Error(b)) => a == &b.to_string(),

			(Global::FromUtf8(a), Global::FromUtf8(b)) => a.to_string() == b.to_string(),
			(Global::FromUtf8(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::FromUtf8(b)) => a == &b.to_string(),

			(Global::StdIo(a), Global::StdIo(b)) => a.to_string() == b.to_string(),
			(Global::StdIo(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::StdIo(b)) => a == &b.to_string(),

			(Global::ProstDecode(a), Global::ProstDecode(b)) => a.to_string() == b.to_string(),
			(Global::ProstDecode(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::ProstDecode(b)) => a == &b.to_string(),

			(Global::ProstEncode(a), Global::ProstEncode(b)) => a.to_string() == b.to_string(),
			(Global::ProstEncode(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::ProstEncode(b)) => a == &b.to_string(),

			(Global::TryFromIntError(a), Global::TryFromIntError(b)) => {
				a.to_string() == b.to_string()
			}
			(Global::TryFromIntError(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::TryFromIntError(b)) => a == &b.to_string(),

			(Global::TryFromCharError(a), Global::TryFromCharError(b)) => {
				a.to_string() == b.to_string()
			}
			(Global::TryFromCharError(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::TryFromCharError(b)) => a == &b.to_string(),

			(Global::TryFromSliceError(a), Global::TryFromSliceError(b)) => {
				a.to_string() == b.to_string()
			}
			(Global::TryFromSliceError(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::TryFromSliceError(b)) => a == &b.to_string(),

			(Global::ParseBool(a), Global::ParseBool(b)) => a.to_string() == b.to_string(),
			(Global::ParseBool(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::ParseBool(b)) => a == &b.to_string(),

			(Global::ParseInt(a), Global::ParseInt(b)) => a.to_string() == b.to_string(),
			(Global::ParseInt(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::ParseInt(b)) => a == &b.to_string(),

			(Global::ParseChar(a), Global::ParseChar(b)) => a.to_string() == b.to_string(),
			(Global::ParseChar(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::ParseChar(b)) => a == &b.to_string(),

			(Global::ParseFloat(a), Global::ParseFloat(b)) => a.to_string() == b.to_string(),
			(Global::ParseFloat(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::ParseFloat(b)) => a == &b.to_string(),

			(Global::ParseAddr(a), Global::ParseAddr(b)) => a.to_string() == b.to_string(),
			(Global::ParseAddr(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::ParseAddr(b)) => a == &b.to_string(),

			(Global::SystemTime(a), Global::SystemTime(b)) => a.to_string() == b.to_string(),
			(Global::SystemTime(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::SystemTime(b)) => a == &b.to_string(),

			(Global::ParseLog(a), Global::ParseLog(b)) => a.to_string() == b.to_string(),
			(Global::ParseLog(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::ParseLog(b)) => a == &b.to_string(),

			(Global::SetLog(a), Global::SetLog(b)) => a.to_string() == b.to_string(),
			(Global::SetLog(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::SetLog(b)) => a == &b.to_string(),

			(Global::Base64Decode(a), Global::Base64Decode(b)) => a.to_string() == b.to_string(),
			(Global::Base64Decode(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::Base64Decode(b)) => a == &b.to_string(),

			(Global::HexDecode(a), Global::HexDecode(b)) => a.to_string() == b.to_string(),
			(Global::HexDecode(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::HexDecode(b)) => a == &b.to_string(),

			(Global::MpscRecv(a), Global::MpscRecv(b)) => a.to_string() == b.to_string(),
			(Global::MpscRecv(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::MpscRecv(b)) => a == &b.to_string(),

			(Global::CrossbeamReceive(a), Global::CrossbeamReceive(b)) => {
				a.to_string() == b.to_string()
			}
			(Global::CrossbeamReceive(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::CrossbeamReceive(b)) => a == &b.to_string(),

			(Global::ChannelReceive(a), Global::ChannelReceive(b)) => {
				a.to_string() == b.to_string()
			}
			(Global::ChannelReceive(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::ChannelReceive(b)) => a == &b.to_string(),

			(Global::ChannelCanceled(a), Global::ChannelCanceled(b)) => {
				a.to_string() == b.to_string()
			}
			(Global::ChannelCanceled(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::ChannelCanceled(b)) => a == &b.to_string(),

			(Global::ChannelSend(a), Global::ChannelSend(b)) => a.to_string() == b.to_string(),
			(Global::ChannelSend(a), Global::Unnamed(b)) => &a.to_string() == b,
			(Global::Unnamed(a), Global::ChannelSend(b)) => a == &b.to_string(),

			_ => false,
		}
	}
}

impl Eq for Global {}

impl From<&Global> for GlobalErrorData {
	fn from(value: &Global) -> Self {
		match value {
			Global::Unnamed(e) => GlobalErrorData::Unnamed(e.clone()),
			Global::CannotBeNone(e) => GlobalErrorData::CannotBeNone(e.clone()),
			Global::BadBinaryFormat(e) => GlobalErrorData::BadBinaryFormat(e.clone()),
			Global::RoutineTimeout(e) => GlobalErrorData::RoutineTimeout(e.clone()),
			Global::TypeIdMismatch(e) => GlobalErrorData::TypeIdMismatch(e.clone()),
			Global::UnexpectedType(e) => GlobalErrorData::UnexpectedType(e.clone()),
			Global::InvalidFormat(e) => GlobalErrorData::InvalidFormat(e.clone()),

			Global::JsonSerde(e) => GlobalErrorData::JsonSerde(e.to_string()),
			Global::BincodeSerde(e) => GlobalErrorData::BincodeSerde(e.to_string()),
			Global::Utf8Error(e) => GlobalErrorData::Utf8Error(e.to_string()),
			Global::FromUtf8(e) => GlobalErrorData::FromUtf8(e.to_string()),
			Global::StdIo(e) => GlobalErrorData::StdIo(e.to_string()),
			Global::ProstDecode(e) => GlobalErrorData::ProstDecode(e.to_string()),
			Global::ProstEncode(e) => GlobalErrorData::ProstEncode(e.to_string()),
			Global::TryFromIntError(e) => GlobalErrorData::TryFromIntError(e.to_string()),
			Global::TryFromCharError(e) => GlobalErrorData::TryFromCharError(e.to_string()),
			Global::TryFromSliceError(e) => GlobalErrorData::TryFromSliceError(e.to_string()),
			Global::ParseBool(e) => GlobalErrorData::ParseBool(e.to_string()),
			Global::ParseInt(e) => GlobalErrorData::ParseInt(e.to_string()),
			Global::ParseChar(e) => GlobalErrorData::ParseChar(e.to_string()),
			Global::ParseFloat(e) => GlobalErrorData::ParseFloat(e.to_string()),
			Global::ParseAddr(e) => GlobalErrorData::ParseAddr(e.to_string()),
			Global::SystemTime(e) => GlobalErrorData::SystemTime(e.to_string()),
			Global::ParseLog(e) => GlobalErrorData::ParseLog(e.to_string()),
			Global::SetLog(e) => GlobalErrorData::SetLog(e.to_string()),
			Global::Base64Decode(e) => GlobalErrorData::Base64Decode(e.to_string()),
			Global::HexDecode(e) => GlobalErrorData::HexDecode(e.to_string()),
			Global::MpscRecv(e) => GlobalErrorData::MpscRecv(e.to_string()),
			Global::CrossbeamReceive(e) => GlobalErrorData::CrossbeamReceive(e.to_string()),
			Global::ChannelCanceled(e) => GlobalErrorData::ChannelCanceled(e.to_string()),
			Global::ChannelReceive(e) => GlobalErrorData::ChannelReceive(e.to_string()),
			Global::ChannelSend(e) => GlobalErrorData::ChannelSend(e.to_string()),
		}
	}
}

impl From<GlobalErrorData> for Global {
	fn from(value: GlobalErrorData) -> Self {
		match value {
			GlobalErrorData::Unnamed(e) => Global::Unnamed(e),
			GlobalErrorData::CannotBeNone(e) => Global::CannotBeNone(e),
			GlobalErrorData::BadBinaryFormat(e) => Global::BadBinaryFormat(e),
			GlobalErrorData::RoutineTimeout(e) => Global::RoutineTimeout(e),
			GlobalErrorData::TypeIdMismatch(e) => Global::TypeIdMismatch(e),
			GlobalErrorData::UnexpectedType(e) => Global::UnexpectedType(e),
			GlobalErrorData::InvalidFormat(e) => Global::InvalidFormat(e),

			GlobalErrorData::JsonSerde(e) => Global::Unnamed(e),
			GlobalErrorData::BincodeSerde(e) => Global::Unnamed(e),
			GlobalErrorData::Utf8Error(e) => Global::Unnamed(e),
			GlobalErrorData::FromUtf8(e) => Global::Unnamed(e),
			GlobalErrorData::StdIo(e) => Global::Unnamed(e),
			GlobalErrorData::ProstDecode(e) => Global::Unnamed(e),
			GlobalErrorData::ProstEncode(e) => Global::Unnamed(e),
			GlobalErrorData::TryFromIntError(e) => Global::Unnamed(e),
			GlobalErrorData::TryFromCharError(e) => Global::Unnamed(e),
			GlobalErrorData::TryFromSliceError(e) => Global::Unnamed(e),
			GlobalErrorData::ParseBool(e) => Global::Unnamed(e),
			GlobalErrorData::ParseInt(e) => Global::Unnamed(e),
			GlobalErrorData::ParseChar(e) => Global::Unnamed(e),
			GlobalErrorData::ParseFloat(e) => Global::Unnamed(e),
			GlobalErrorData::ParseAddr(e) => Global::Unnamed(e),
			GlobalErrorData::SystemTime(e) => Global::Unnamed(e),
			GlobalErrorData::ParseLog(e) => Global::Unnamed(e),
			GlobalErrorData::SetLog(e) => Global::Unnamed(e),
			GlobalErrorData::Base64Decode(e) => Global::Unnamed(e),
			GlobalErrorData::HexDecode(e) => Global::Unnamed(e),
			GlobalErrorData::MpscRecv(e) => Global::Unnamed(e),
			GlobalErrorData::CrossbeamReceive(e) => Global::Unnamed(e),
			GlobalErrorData::ChannelCanceled(e) => Global::Unnamed(e),
			GlobalErrorData::ChannelReceive(e) => Global::Unnamed(e),
			GlobalErrorData::ChannelSend(e) => Global::Unnamed(e),
		}
	}
}

impl Serialize for Global {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let data = GlobalErrorData::from(self);
		data.serialize(serializer)
	}
}

impl<'a> Deserialize<'a> for Global {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'a>,
	{
		let data: GlobalErrorData = Deserialize::deserialize(deserializer)?;
		Ok(data.into())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn global_equal_works() {
		let err = Global::Unnamed("test".to_owned());
		assert_eq!(err, err.clone());

		let err = Global::CannotBeNone(CannotBeNone("test".to_owned()));
		assert_eq!(err, err.clone());

		let value: Result<u64, _> = "test".parse();
		let err: Global = value.unwrap_err().into();
		assert_eq!(err, err.clone());

		let value = hex::decode("test");
		let err: Global = value.unwrap_err().into();
		assert_eq!(err, err.clone());
	}
}
