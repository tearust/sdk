use crate::actor_txns::error::{Result, TxnError};
use crate::tapp::{Account, TokenId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// These are allow operation that user authorize the tapp to
/// When user login a tapp, these allow ops will be listed
/// If user agree, login continue, if not, login cancelled
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone)]
pub enum AllowedOp {
	Read,         // Read the account balance
	Move,         // To move the fund to other account or my deposit account
	Withdraw,     // Withdraw the fund from layer2 to my layer1 account (not others)
	Consume,      // Pay the bill of using this app (not other apps)
	BondingCurve, // Allow buy and sell bonding curve token
	CrossMove,    // Allow moving fund between TApps (different tokenids)
}

#[doc(hidden)]
#[derive(Debug, Serialize, Default, Deserialize, PartialEq, Eq, Clone)]
pub struct TokenAuthOp {
	pub token_id: TokenId,
	pub acct: Account,
	pub read: bool,
	pub mov: bool,
	pub withdraw: bool,
	pub consume: bool,
	pub bonding_curve: bool,
}

impl TokenAuthOp {
	pub fn new(token_id: TokenId, acct: Account, auth_expr: &str) -> Self {
		TokenAuthOp {
			token_id,
			acct,
			read: auth_expr.contains("read"),
			mov: auth_expr.contains("move"),
			withdraw: auth_expr.contains("withdraw"),
			consume: auth_expr.contains("consume"),
			bonding_curve: auth_expr.contains("bonding_curve"),
		}
	}

	/// Check auth method.
	pub fn check_auth(
		&self,
		ask_acct: Account,
		ask_token_id: TokenId,
		ask_op: AllowedOp,
	) -> Result<()> {
		let check_failed_err = TxnError::AuthCheckFailed(ask_token_id, ask_acct, ask_op);
		if !self.is_token_id_match(ask_token_id) || !self.is_account_match(ask_acct) {
			return Err(check_failed_err.into());
		}

		let allowed_ops: HashSet<AllowedOp> = self.clone().into();
		if allowed_ops.contains(&ask_op) {
			return Ok(());
		}

		Err(check_failed_err.into())
	}

	fn is_token_id_match(&self, ask_token_id: TokenId) -> bool {
		if self.token_id == ask_token_id {
			return true;
		}

		// TODO: add tappstore_id
		// this is the hidden token id allowed by default
		// ask_token_id == tappstore_id()?
		true
	}

	fn is_account_match(&self, ask_acct: Account) -> bool {
		self.acct == ask_acct
	}
}

impl From<TokenAuthOp> for HashSet<AllowedOp> {
	fn from(val: TokenAuthOp) -> Self {
		let mut allowed_set = HashSet::new();
		if val.read {
			allowed_set.insert(AllowedOp::Read);
		}
		if val.mov {
			allowed_set.insert(AllowedOp::Move);
		}
		if val.consume {
			allowed_set.insert(AllowedOp::Consume);
		}
		if val.withdraw {
			allowed_set.insert(AllowedOp::Withdraw);
		}
		if val.bonding_curve {
			allowed_set.insert(AllowedOp::BondingCurve);
		}
		allowed_set
	}
}
