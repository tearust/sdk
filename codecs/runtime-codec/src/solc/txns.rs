use ethereum_types::{H160, U256};
use serde::{Deserialize, Serialize};

use crate::actor_txns::Tsid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncTxn {
	pub seq_number: u64,
	pub txn_type: TxnType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TxnType {
	SignWithdraw(Vec<u8>),
	SignMintCml(Vec<u8>),
	SendWithdraw {
		records: Vec<u8>,
		signatures: Vec<u8>,
		nonce: U256,
		tsid: Tsid,
	},
	SendMintCml {
		records: Vec<u8>,
		signatures: Vec<u8>,
		nonce: U256,
		tsid: Tsid,
	},
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockRecordTrans {
	pub token: H160,
	pub recipient: H160,
	pub amount: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintCmlRecordTrans {
	pub to: H160,
	pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleSign {
	pub signature: Vec<u8>,
	pub nonce: U256,
}

impl SingleSign {
	pub fn new(signature: Vec<u8>, nonce: U256) -> Self {
		Self { signature, nonce }
	}
}
