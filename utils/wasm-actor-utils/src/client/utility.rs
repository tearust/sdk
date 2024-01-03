use crate::client::error::Result;
use tea_runtime_codec::tapp::Account;
use tea_sdk::ResultExt;

/// Parse address string to Ethereum account address (H160)
pub fn parse_to_acct(address: &str) -> Result<Account> {
	address.parse().err_into()
}
