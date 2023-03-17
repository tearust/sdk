use error::Result;
use primitive_types::{H160, U256};
use serde::{Deserialize, Serialize};
use std::{
	fmt::{Debug, Display, Formatter},
	sync::Arc,
};
use tea_sdk::serde::TypeId;

pub mod cml;
pub mod entity;
pub mod error;
pub mod event;
pub mod fluencer;
pub mod machine;
pub mod ra;
pub mod seat;
pub mod statement;
pub mod sys;
pub mod tapp;
pub mod version;

pub type Hash = [u8; 32];
pub type Ts = u128;
pub type ReplicaId = [u8; 32];
pub type BlockNumber = u64;

pub type Account = H160;
pub type Balance = U256;
pub const CENTS: Balance = U256([10_000_000_000_000_000, 0, 0, 0]);
pub const DOLLARS: Balance = U256([1_000_000_000_000_000_000, 0, 0, 0]);

/// When use TOKEN_ID_TAPPSTORE, this is not for any tapp.
/// it is just a TEA account for a user transfer
/// between layer one and layer two
pub const MOCK_TOKEN_ID_TAPPSTORE: TokenId = TokenId(H160::zero());

/// Short format of timestamp (from chrono timestamp), to distinguish between transactions
///  that may send multiple times
pub type TimestampShort = i64;

/// AuthKey is silimar to session key. When end user login he
/// need to sign a AuthOps data strcuture, in this AuthOps
/// he agree this session (IDed by this AuthKey) can do what
/// operation on his account.
/// The AuthKey is the hashmap key to the AuthOps
/// Currently in epoch7 Dec 2021, the AuthKey is the
/// same as AesKey
pub type AuthKey = u128;
/// God mode auth key only allowed in system actors
pub const GOD_MODE_AUTH_KEY: AuthKey = u128::MAX;
pub const RECEIPTING_AUTH_KEY: AuthKey = 1;
pub const PUBLIC_RESERVED_ACCOUNT: Account = H160([1_u8; 20]);

/// tokenId is actually the TappId.
/// When a Tapp is created, it is issued a TApp Id from Layer one.
/// Then the user can topup (transfer from layer one
/// to layer two) using this TokenId
#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct TokenId(pub H160);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, PartialEq, Eq, Hash)]
pub enum AccountId {
	User(H160),
	App(H160),
	Other(Arc<[u8]>),
}

impl Display for AccountId {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		Debug::fmt(&self, f)
	}
}

impl From<[u8; 20]> for TokenId {
	fn from(v: [u8; 20]) -> Self {
		TokenId(H160(v))
	}
}

impl From<H160> for TokenId {
	fn from(v: H160) -> Self {
		TokenId(v)
	}
}

impl Debug for TokenId {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "0x{}", hex::encode(self.0))
	}
}

impl TokenId {
	pub fn to_hex(&self) -> String {
		format!("{:?}", self.0)
	}

	pub fn from_hex<T: AsRef<str>>(s: T) -> Result<Self> {
		let inner: H160 = s
			.as_ref()
			.parse()
			.map_err(|_| crate::tapp::error::Errors::ParseAddressError)?;
		Ok(inner.into())
	}
}
