use crate::client::Result;
use tea_codec::ResultExt;
use tea_runtime_codec::tapp::Account;

/// Parse address string to Ethereum account address (H160)
pub fn parse_to_acct(address: &str) -> Result<Account> {
	address.parse().err_into()
}
