use crate::Result;
use tapp_common::Account;
use tea_codec::ResultExt;

pub fn parse_to_acct(address: &str) -> Result<Account> {
    address.parse().err_into()
}
