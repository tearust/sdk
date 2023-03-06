use crate::Result;
use tea_codec::ResultExt;
use tea_tapp_common::Account;

pub fn parse_to_acct(address: &str) -> Result<Account> {
	address.parse().err_into()
}
