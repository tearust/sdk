#![feature(min_specialization)]
pub mod error;

use error::Result;
use ethereum_types::Address;
use tea_sdk::ResultExt;

#[macro_use]
extern crate serde_derive;
extern crate tea_codec as tea_sdk;

pub mod queries;
pub mod txns;

pub type BlockNumber = u64;
pub type CmlId = u64;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContractAddresses {
    pub lock: String,
    pub storage: String,
    pub maintainer: String,
    pub token_vesting: String,
    pub erc721: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EventType {
    Topup,
    Withdraw,
    ValidatorChanged,
    TransferCml,
}

impl ContractAddresses {
    pub fn lock_address(&self) -> Result<Address> {
        self.string_to_address(&self.lock, "lock")
    }

    pub fn storage_address(&self) -> Result<Address> {
        self.string_to_address(&self.storage, "storage")
    }

    pub fn maintainer_address(&self) -> Result<Address> {
        self.string_to_address(&self.maintainer, "maintainer")
    }

    pub fn token_vesting_address(&self) -> Result<Address> {
        self.string_to_address(&self.token_vesting, "token_vesting")
    }

    pub fn erc721_address(&self) -> Result<Address> {
        self.string_to_address(&self.erc721, "erc721")
    }

    fn string_to_address(&self, addr: &str, _name: &str) -> Result<Address> {
        addr.parse().err_into()
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Topup => write!(f, "topup"),
            EventType::Withdraw => write!(f, "withdraw"),
            EventType::ValidatorChanged => write!(f, "validator changed"),
            EventType::TransferCml => write!(f, "transfer cml"),
        }
    }
}
