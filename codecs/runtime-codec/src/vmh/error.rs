use crate::runtime::error::RuntimeCodec;
use prost::{DecodeError, EncodeError};
use tea_sdk::define_scope;
use thiserror::Error;

pub type VmhResult<T> = Result<T>;
pub type VmhError = Error;

define_scope! {
	VmhCodec: RuntimeCodec {
		GeneralVmh;
		SocketSendU64;
		SocketSendLoop;
		SocketRecvU64;
		SocketRecvLoop;
		SocketClientDisconnected;
		SocketServerClosed;
		SocketRecvSizeMismatched;
		SocketSendSizeMismatched;
		SocketNix;
		QuitReceiverLoop;
		OperationInvalid;
		SenderOperationExists;
		InboundNet;
		OutboundNet;
		IntercomActorNotSupported;
		IntercomRequestRejected;
		TableAccess => TableAccess, @Display, @Debug;
		PersistCheck => PersistCheck, @Display, @Debug;
		EncodeError => Encode, @Display, @Debug;
		DecodeError => Encode, @Display, @Debug;
		Errors => VmhGeneral, @Display, @Debug;
	}
}

#[derive(Error, Debug)]
pub enum TableAccess {
	#[error("Failed to get row at {0} in table {1}")]
	GetRow(usize, String),

	#[error("Failed to convert table {0} to array")]
	ConvertToArray(String),

	#[error("Failed to get table {0}")]
	GetTable(String),
}

#[derive(Error, Debug)]
pub enum PersistCheck {
	#[error("Prefix length to long, expect is {0} actual is {1}")]
	PrefixTooLong(usize, usize),
	#[error("Key {0:?} is too short to remove prefix")]
	KeyTooShort(Vec<u8>),
	#[error("Prefix length mismatched, expect is {0} actual is {1}")]
	PrefixLengthMismatch(usize, usize),
}

#[derive(Error, Debug)]
pub enum Errors {
	#[error("Unknown built-in env {0}")]
	UnknownBuiltInEnv(String),

	#[error("Unknown app command {0}")]
	UnknownAppCommand(String),

	#[error("Unknown upgrade type {0}")]
	UnknownUpgradeType(String),

	#[error("Txn hash file not exists {0}")]
	TxnHashFileNotExists(i64),
}
