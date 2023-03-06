use tea_actor_txns::tsid::Tsid;
use tea_tapp_common::{Account, Balance, TokenId};

use crate::error::{Result, TableAccess};
use crate::state::TsidReadable;
use std::collections::HashMap;
use std::time::Duration;

/// Definitions for dump purposes, should not be used in production mode.
pub const ROW_KEY_OF_EMPTY_TABLE: &str = "<headers>";

#[derive(Debug, Serialize, Deserialize)]
pub struct DumpTxnSeqRequest {
	pub show_conveyor_mutable: bool,
	pub show_conveyor_executed: bool,
	pub show_history: bool,
	pub show_exec_time: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DumpTxnSeqResponse {
	pub exec_cursor: Option<TsidReadable>,
	pub conveyor_mutable: Vec<TsidReadable>,
	pub conveyor_executed: Vec<TsidReadable>,
	pub history_txns: Vec<(TsidReadable, Duration)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DumpGlobalStateRequest {
	pub show_tapp_general: bool,
	pub show_auth_key: bool,
	pub show_session_key: bool,
	pub show_failed_payments: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DumpGlobalStateIntermediate {
	pub last_exec_tsid: TsidReadable,
	pub tapp_store_key: Option<String>,
	pub aes_keys: Vec<(TokenId, String)>,
	pub failed_payments: Vec<(TokenId, Vec<u8>)>,
	pub consume_account_keys: Vec<(TokenId, Vec<u8>)>,
	pub auth_keys: Vec<(u128, Vec<u8>)>,
	pub session_keys: Vec<(TokenId, Account, Vec<u8>)>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DumpGlobalStateResponse {
	pub last_exec_tsid: TsidReadable,
	pub tapp_store_key: Option<String>,
	/// items fields:
	/// - token id
	/// - key
	pub aes_keys: Vec<(String, String)>,
	/// items fields:
	/// - token id
	/// - payments
	pub failed_payments: Vec<(String, String)>,
	/// items fields:
	/// - token id
	/// - key
	pub consume_account_keys: Vec<(String, String)>,
	/// items fields:
	/// - auth id
	/// - auth key
	pub auth_keys: Vec<(u128, String)>,
	/// items fields:
	/// - token id
	/// - account id
	/// - session key
	pub session_keys: Vec<(String, String, String)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DumpTappStateRequest {
	pub token_id: Option<String>,
	pub show_tea_balances: bool,
	pub show_token_balances: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DumpTappStateIntermediate {
	pub root_tsid: Option<Tsid>,
	pub tea_tsid: Option<Tsid>,
	pub tea_balances: Vec<(TokenId, Vec<(Account, Balance)>)>,
	pub tea_deposit_balances: Vec<(TokenId, Vec<(Account, Balance)>)>,
	pub token_balances: Vec<(TokenId, Vec<(Account, Balance)>)>,
	pub token_reserved_balances: Vec<(TokenId, Vec<(Account, Balance)>)>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DumpTappStateResponse {
	pub root_tsid: Option<TsidReadable>,
	pub tea_tsid: Option<TsidReadable>,
	pub tea_balances: Vec<(String, Vec<(String, Balance)>)>,
	pub tea_deposit_balances: Vec<(String, Vec<(String, Balance)>)>,
	pub token_balances: Vec<(String, Vec<(String, Balance)>)>,
	pub token_reserved_balances: Vec<(String, Vec<(String, Balance)>)>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DumpRoundTableRequest {}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GetRoundTableRequest {
	pub uuid: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DumpRoundTableResponse {
	pub desired_count: u8,
	pub min_count: u8,
	pub replica_ids: Vec<(String, String)>,
	pub update_tsid: TsidReadable,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ListPeersRequest {}
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ListPeersResponse {
	pub my_peer_id: String,
	pub peers: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SaveStateRequest {}
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SaveStateResponse {}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DumpNodeProfileRequest {}
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DumpNodeProfileGetResult {
	pub uuid: String,
}
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DumpNodeProfileResponse {
	pub ephemeral_public_key: String,
	pub tea_id: String,
	pub conn_id: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DumpGluedbDataRequest {
	pub token_id: String,
	pub max_rows: Option<u64>,
}
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DumpGluedbDataResponse {
	pub tables: HashMap<String, serde_json::Value>,
}
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DumpGluedbDataSorted {
	pub tables: Vec<(String, serde_json::Value)>,
}

impl From<DumpGluedbDataResponse> for DumpGluedbDataSorted {
	fn from(value: DumpGluedbDataResponse) -> Self {
		let mut sorted = DumpGluedbDataSorted {
			tables: value.tables.into_iter().collect(),
		};
		sorted.tables.sort_by(|(a, _), (b, _)| a.cmp(b));
		sorted
	}
}

impl DumpGluedbDataResponse {
	pub fn first_row(&self, table: &str) -> Result<&serde_json::Value> {
		self.get_row_at(table, 0)
	}

	pub fn get_row_at(&self, table: &str, index: usize) -> Result<&serde_json::Value> {
		self.get_table_rows(table)?
			.get(index)
			.ok_or_else(|| TableAccess::GetRow(index, table.to_string()).into())
	}

	pub fn get_table_rows(&self, table: &str) -> Result<&Vec<serde_json::Value>> {
		self.get_table(table)?
			.as_array()
			.ok_or_else(|| TableAccess::ConvertToArray(table.to_string()).into())
	}

	pub fn get_table(&self, table: &str) -> Result<&serde_json::Value> {
		self.tables
			.get(table)
			.ok_or_else(|| TableAccess::GetTable(table.to_string()).into())
	}

	pub fn is_table_empty(&self, table: &str) -> Result<bool> {
		let rows = self.get_table_rows(table)?;
		if rows.len() != 1 {
			return Ok(false);
		}
		Ok(rows[0].get(ROW_KEY_OF_EMPTY_TABLE).is_some())
	}
}

impl From<DumpTappStateIntermediate> for DumpTappStateResponse {
	fn from(val: DumpTappStateIntermediate) -> Self {
		DumpTappStateResponse {
			root_tsid: val.root_tsid.map(|v| v.into()),
			tea_tsid: val.tea_tsid.map(|v| v.into()),
			tea_balances: to_visiable_balances(&val.tea_balances),
			tea_deposit_balances: to_visiable_balances(&val.tea_deposit_balances),
			token_balances: to_visiable_balances(&val.token_balances),
			token_reserved_balances: to_visiable_balances(&val.token_reserved_balances),
		}
	}
}

type VisiableItem = (TokenId, Vec<(Account, Balance)>);
fn to_visiable_balances(balance_map: &[VisiableItem]) -> Vec<(String, Vec<(String, Balance)>)> {
	balance_map
		.iter()
		.map(|(token_id, balances)| {
			(
				format!("{token_id:?}"),
				balances
					.iter()
					.map(|(account, balance)| (hex::encode(account), *balance))
					.collect(),
			)
		})
		.collect()
}

impl From<DumpGlobalStateIntermediate> for DumpGlobalStateResponse {
	fn from(val: DumpGlobalStateIntermediate) -> Self {
		DumpGlobalStateResponse {
			last_exec_tsid: val.last_exec_tsid,
			tapp_store_key: val.tapp_store_key,
			aes_keys: val
				.aes_keys
				.into_iter()
				.map(|(token_id, key)| (format!("{token_id:?}"), key))
				.collect(),
			failed_payments: val
				.failed_payments
				.into_iter()
				.map(|(token_id, payment)| (format!("{token_id:?}"), hex::encode(payment)))
				.collect(),
			consume_account_keys: val
				.consume_account_keys
				.into_iter()
				.map(|(token_id, account)| (format!("{token_id:?}"), hex::encode(account)))
				.collect(),
			auth_keys: val
				.auth_keys
				.into_iter()
				.map(|(id, key)| (id, hex::encode(key)))
				.collect(),
			session_keys: val
				.session_keys
				.into_iter()
				.map(|(token_id, account, key)| {
					(
						format!("{token_id:?}"),
						format!("{account:?}"),
						hex::encode(key),
					)
				})
				.collect(),
		}
	}
}
