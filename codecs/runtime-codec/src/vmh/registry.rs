use crate::tapp::{
	cml::{CmlId, CmlIntrinsic},
	Account,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GenesisSettings {
	pub startup_nodes: Vec<(CmlId, String)>,
	pub tappstore_owner: Account,
	pub knowing_cmls: Vec<CmlIntrinsic>,
}
