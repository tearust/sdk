use crate::actor_txns::pre_args::ArgSlots;
use crate::actor_txns::{error::Result, txn_hash};
use crate::tapp::Hash;
use serde::{Deserialize, Serialize};
use tea_sdk::ResultExt;

use super::TxnSerial;

#[doc(hidden)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FullTxn {
	pub txn_bytes: Vec<u8>,
	pub args: Option<ArgSlots>,
}

impl FullTxn {
	pub fn new(txn_bytes: Vec<u8>, args: Option<ArgSlots>) -> Self {
		Self { txn_bytes, args }
	}

	pub fn new_no_args(txn_bytes: Vec<u8>) -> Self {
		Self {
			txn_bytes,
			args: None,
		}
	}

	pub fn size(&self) -> usize {
		self.txn_bytes.len()
			+ match self.args.as_ref() {
				Some(args) => args.size() + 1,
				None => 1,
			}
	}

	pub fn txn_hash(&self) -> Result<Hash> {
		let txn_serial: TxnSerial = tea_codec::deserialize(&self.txn_bytes)?;
		txn_hash(txn_serial.hasy_bytes()?.as_slice()).err_into()
	}

	pub fn args_hash(&self) -> Result<Option<Hash>> {
		self.args.as_ref().map(|v| v.hash()).transpose().err_into()
	}
}
