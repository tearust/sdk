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

use crate::serde::error::{InvalidFormat, TypeIdMismatch, UnexpectedType};

#[derive(thiserror::Error, Debug)]
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
	JsonSerde(#[from] serde_json::Error),

	#[error(transparent)]
	BincodeSerde(#[from] bincode::Error),

	#[error(transparent)]
	Utf8Error(#[from] Utf8Error),

	#[error(transparent)]
	FromUtf8(#[from] FromUtf8Error),

	#[error(transparent)]
	StdIo(#[from] std::io::Error),

	#[error(transparent)]
	ProstDecode(#[from] prost::DecodeError),

	#[error(transparent)]
	ProstEncode(#[from] prost::EncodeError),

	#[error(transparent)]
	TryFromIntError(#[from] TryFromIntError),

	#[error(transparent)]
	TryFromCharError(#[from] TryFromCharError),

	#[error(transparent)]
	TryFromSliceError(#[from] TryFromSliceError),

	#[error(transparent)]
	ParseBool(#[from] ParseBoolError),

	#[error(transparent)]
	ParseInt(#[from] ParseIntError),

	#[error(transparent)]
	ParseChar(#[from] ParseCharError),

	#[error(transparent)]
	ParseFloat(#[from] ParseFloatError),

	#[error(transparent)]
	ParseAddr(#[from] AddrParseError),

	#[error(transparent)]
	SystemTime(#[from] SystemTimeError),

	#[error(transparent)]
	ParseLog(#[from] ParseLevelError),

	#[error(transparent)]
	SetLog(#[from] SetLoggerError),

	#[error(transparent)]
	Base64Decode(#[from] base64::DecodeError),

	#[error(transparent)]
	HexDecode(#[from] FromHexError),

	#[error(transparent)]
	MpscRecv(#[from] std::sync::mpsc::RecvError),

	#[error(transparent)]
	CrossbeamReceive(#[from] crossbeam_channel::RecvError),

	#[error(transparent)]
	ChannelReceive(#[from] futures::channel::mpsc::TryRecvError),

	#[error(transparent)]
	ChannelCanceled(#[from] futures::channel::oneshot::Canceled),

	#[error(transparent)]
	ChannelSend(#[from] futures::channel::mpsc::SendError),
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
