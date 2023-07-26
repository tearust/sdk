use crate::client::error::Result;
use crate::enclave::actors::statemachine;
// use tea_codec::{deserialize, serialize};
use tea_runtime_codec::tapp::{Account, Balance, TokenId, Ts};

pub async fn fetch_tea_balance(token_id: TokenId, acct: Account) -> Result<(Ts, Balance)> {
	let balance = statemachine::query_tea_balance(token_id, acct).await?;
	let latest_tsid = statemachine::query_state_tsid().await?;

	Ok((latest_tsid.ts, balance))
}

pub async fn fetch_tea_deposit(token_id: TokenId, acct: Account) -> Result<(Ts, Balance)> {
	let balance = statemachine::query_tea_deposit_balance(token_id, acct).await?;
	let latest_tsid = statemachine::query_state_tsid().await?;

	Ok((latest_tsid.ts, balance))
}

pub fn is_system_actor(from_actor: &str) -> bool {
	from_actor.starts_with("com.tea.")
}
