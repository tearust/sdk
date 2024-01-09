use std::{
	array::TryFromSliceError,
	char::{ParseCharError, TryFromCharError},
	net::AddrParseError,
	num::{ParseFloatError, ParseIntError, TryFromIntError},
	str::{ParseBoolError, Utf8Error},
	string::FromUtf8Error,
	time::SystemTimeError,
};

use hex::FromHexError;
use log::{ParseLevelError, SetLoggerError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::actorx::*;
use crate::serde::error::{InvalidFormat, TypeIdMismatch, UnexpectedType};

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum Global {
	#[error("Global error: {0}")]
	Unnamed(String),

	#[error(transparent)]
	CannotBeNone(#[from] CannotBeNone),

	#[error(transparent)]
	BadBinaryFormat(#[from] BadBinaryFormat),

	#[error(transparent)]
	RoutineTimeout(#[from] RoutineTimeout),

	#[error(transparent)]
	TypeIdMismatch(#[from] TypeIdMismatch),

	#[error(transparent)]
	UnexpectedType(#[from] UnexpectedType),

	#[error(transparent)]
	InvalidFormat(#[from] InvalidFormat),

	#[error(transparent)]
	GasFeeExhausted(#[from] GasFeeExhausted),

	#[error(transparent)]
	WorkerCrashed(#[from] WorkerCrashed),

	#[error(transparent)]
	AccessNotPermitted(#[from] AccessNotPermitted),

	#[error(transparent)]
	ActorNotExist(#[from] ActorNotExist),

	#[error(transparent)]
	NotSupported(#[from] NotSupported),

	#[error(transparent)]
	ActorHostDropped(#[from] ActorHostDropped),

	#[error(transparent)]
	ActorDeactivating(#[from] ActorDeactivating),

	#[error(transparent)]
	InvocationTimeout(#[from] InvocationTimeout),

	#[error(transparent)]
	ChannelReceivingTimeout(#[from] ChannelReceivingTimeout),

	#[error("Json serde error: {0}")]
	JsonSerde(String),

	#[error("Bincode error: {0}")]
	BincodeSerde(String),

	#[error("From utf8 error: {0}")]
	FromUtf8(String),

	#[error("IO error: {0}")]
	StdIo(String),

	#[error("Prost decode error: {0}")]
	ProstDecode(String),

	#[error("Prost encode error: {0}")]
	ProstEncode(String),

	#[error("Try from int error: {0}")]
	TryFromIntError(String),

	#[error("Try from char error: {0}")]
	TryFromCharError(String),

	#[error("Try from slice error: {0}")]
	TryFromSliceError(String),

	#[error("Parse bool error: {0}")]
	ParseBool(String),

	#[error("Parse int error: {0}")]
	ParseInt(String),

	#[error("Parse char error: {0}")]
	ParseChar(String),

	#[error("Parse float error: {0}")]
	ParseFloat(String),

	#[error("Parse address error: {0}")]
	ParseAddr(String),

	#[error("System time error: {0}")]
	SystemTime(String),

	#[error("Parse log level error: {0}")]
	ParseLevel(String),

	#[error("Set logger error: {0}")]
	SetLog(String),

	#[error("Base64 decode error: {0}")]
	Base64Decode(String),

	#[error("Hex decode error: {0}")]
	HexDecode(String),

	#[error("Mpsc receiving error: {0}")]
	MpscRecv(String),

	#[error("Crossbeam receiving error: {0}")]
	CrossbeamReceive(String),

	#[error("Channel receiving error: {0}")]
	ChannelReceive(String),

	#[error("Channel canceled error: {0}")]
	ChannelCanceled(String),

	#[error("Channel send error: {0}")]
	ChannelSend(String),
}

impl From<serde_json::Error> for Global {
	fn from(e: serde_json::Error) -> Self {
		Global::JsonSerde(e.to_string())
	}
}

impl From<bincode::Error> for Global {
	fn from(e: bincode::Error) -> Self {
		Global::BincodeSerde(e.to_string())
	}
}

impl From<Utf8Error> for Global {
	fn from(e: Utf8Error) -> Self {
		Global::FromUtf8(e.to_string())
	}
}

impl From<FromUtf8Error> for Global {
	fn from(e: FromUtf8Error) -> Self {
		Global::FromUtf8(e.to_string())
	}
}

impl From<std::io::Error> for Global {
	fn from(e: std::io::Error) -> Self {
		Global::StdIo(e.to_string())
	}
}

impl From<prost::DecodeError> for Global {
	fn from(e: prost::DecodeError) -> Self {
		Global::ProstDecode(e.to_string())
	}
}

impl From<prost::EncodeError> for Global {
	fn from(e: prost::EncodeError) -> Self {
		Global::ProstEncode(e.to_string())
	}
}

impl From<TryFromIntError> for Global {
	fn from(e: TryFromIntError) -> Self {
		Global::TryFromIntError(e.to_string())
	}
}

impl From<TryFromCharError> for Global {
	fn from(e: TryFromCharError) -> Self {
		Global::TryFromCharError(e.to_string())
	}
}

impl From<TryFromSliceError> for Global {
	fn from(e: TryFromSliceError) -> Self {
		Global::TryFromSliceError(e.to_string())
	}
}

impl From<ParseBoolError> for Global {
	fn from(e: ParseBoolError) -> Self {
		Global::ParseBool(e.to_string())
	}
}

impl From<ParseIntError> for Global {
	fn from(e: ParseIntError) -> Self {
		Global::ParseInt(e.to_string())
	}
}

impl From<ParseCharError> for Global {
	fn from(e: ParseCharError) -> Self {
		Global::ParseChar(e.to_string())
	}
}

impl From<ParseFloatError> for Global {
	fn from(e: ParseFloatError) -> Self {
		Global::ParseFloat(e.to_string())
	}
}

impl From<AddrParseError> for Global {
	fn from(e: AddrParseError) -> Self {
		Global::ParseAddr(e.to_string())
	}
}

impl From<SystemTimeError> for Global {
	fn from(e: SystemTimeError) -> Self {
		Global::SystemTime(e.to_string())
	}
}

impl From<ParseLevelError> for Global {
	fn from(e: ParseLevelError) -> Self {
		Global::ParseLevel(e.to_string())
	}
}

impl From<SetLoggerError> for Global {
	fn from(e: SetLoggerError) -> Self {
		Global::SetLog(e.to_string())
	}
}

impl From<base64::DecodeError> for Global {
	fn from(e: base64::DecodeError) -> Self {
		Global::Base64Decode(e.to_string())
	}
}

impl From<FromHexError> for Global {
	fn from(e: FromHexError) -> Self {
		Global::HexDecode(e.to_string())
	}
}

impl From<std::sync::mpsc::RecvError> for Global {
	fn from(e: std::sync::mpsc::RecvError) -> Self {
		Global::MpscRecv(e.to_string())
	}
}

impl From<crossbeam_channel::RecvError> for Global {
	fn from(e: crossbeam_channel::RecvError) -> Self {
		Global::CrossbeamReceive(e.to_string())
	}
}

impl From<futures::channel::mpsc::TryRecvError> for Global {
	fn from(value: futures::channel::mpsc::TryRecvError) -> Self {
		Global::ChannelReceive(value.to_string())
	}
}

impl From<futures::channel::oneshot::Canceled> for Global {
	fn from(e: futures::channel::oneshot::Canceled) -> Self {
		Global::ChannelCanceled(e.to_string())
	}
}

impl From<futures::channel::mpsc::SendError> for Global {
	fn from(e: futures::channel::mpsc::SendError) -> Self {
		Global::ChannelSend(e.to_string())
	}
}

#[derive(Error, Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[error("Value \"{0}\" cannot be none")]
pub struct CannotBeNone(pub String);

#[derive(Error, Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[error("Bad binary format")]
pub struct BadBinaryFormat;

#[derive(Error, Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[error("Routine timeout at checkpoint {0}")]
pub struct RoutineTimeout(pub String);

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_unnamed() {
		let err = Global::Unnamed("test".to_owned());
		assert_eq!(err.to_string(), "Global error: test");

		let err = Global::CannotBeNone(CannotBeNone("test".to_owned()));
		assert_eq!(err.to_string(), "Value \"test\" cannot be none");
	}

	#[test]
	fn nested_error() {
		#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
		pub enum MyError {
			#[error(transparent)]
			Global(#[from] Global),

			#[error("others error: {0}")]
			Others(String),
		}

		let err = MyError::Global(Global::UnexpectedType(UnexpectedType("my type".to_owned())));
		assert!(matches!(err, MyError::Global(Global::UnexpectedType(_))));
		assert_eq!(err.to_string(), "Type id \"my type\" is not supported here");
	}
}
