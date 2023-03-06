use crate::error::Result;
use crate::help;
use crate::request;

use prost::Message;
use serde_json::json;
use std::str::FromStr;
use tapp_common::{Balance, TokenId};
use tappstore_actor_codec::txns::TappstoreTxn;
use tappstore_actor_codec::CheckUserSessionRequest;
use tappstore_actor_codec::CommonSqlQueryRequest;
use tappstore_actor_codec::FetchAccountAssetRequest;
use tappstore_actor_codec::FetchAllowanceRequest;
use tappstore_actor_codec::FindExecutedTxnRequest;
use tappstore_actor_codec::QueryTeaBalanceRequest;
use tappstore_actor_codec::QueryTeaDepositRequest;
use tea_codec::OptionExt;
use tea_codec::{deserialize, serialize};
use vmh_codec::message::{
	encode_protobuf,
	structs_proto::{replica, tappstore},
};
use wasm_actor_utils::actors::enclave::get_my_tea_id;
use wasm_actor_utils::actors::util as actor_util;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
	pub tapp_id_b64: String,
	pub address: String,
	/// Base64 encoded
	pub data: String,
	/// Base64 encoded
	pub signature: String,
	pub pk: String,
	pub uuid: String,
}
pub async fn txn_login(payload: Vec<u8>, from_actor: String) -> Result<Vec<u8>> {
	let req: LoginRequest = serde_json::from_slice(&payload)?;
	info!("login request action... {:?}", req);
	let _txn_uuid = req.uuid.to_string();

	let txn = TappstoreTxn::GenSessionKey {
		token_id: TokenId::from_hex(&req.tapp_id_b64)?,
		acct: actor_util::str_to_h160(&req.address)?,
		pk: base64::decode(&req.pk)?,
		data: req.data.clone(),
		signature: req.signature.clone(),
		tea_id: get_my_tea_id().await?,
	};

	request::send_tappstore_txn(
		&from_actor,
		"Login",
		&req.uuid,
		tea_codec::serialize(&req)?,
		txn,
		vec![],
		None,
	)
	.await?;

	help::result_ok()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuerySessionKeyRequest {
	pub tapp_id_b64: String,
	pub address: String,
	pub uuid: String,
}
pub async fn query_session_key(payload: Vec<u8>, from_actor: String) -> Result<Vec<u8>> {
	let req: QuerySessionKeyRequest = serde_json::from_slice(&payload)?;
	let uuid = req.uuid;

	request::send_tappstore_query(
		&from_actor,
		CheckUserSessionRequest {
			account: req.address.parse()?,
			token_id: TokenId::from_hex(&req.tapp_id_b64)?,
		},
		move |r| {
			Box::pin(async move {
				let aes_key = &r.aes_key;
				let auth_key = r
					.auth_key
					.map(|v| serialize(&v))
					.transpose()?
					.unwrap_or_default();

				let auth_b64 = base64::encode(auth_key);
				info!("save auth_b64 => {:?}", auth_b64);
				info!("save aes_key => {:?}", aes_key);

				help::save_session_key(
					auth_b64.clone(),
					&r.token_id.to_hex(),
					&format!("{:?}", r.account),
				)
				.await?;
				help::save_aes_key(aes_key.to_vec(), &r.token_id.to_hex()).await?;

				let x = serde_json::json!({
					"auth_key": auth_b64,
				});
				help::cache_json_with_uuid(&uuid, x).await?;
				Ok(())
			})
		},
	)
	.await?;

	help::result_ok()
}

pub async fn check_auth(tapp_id_hex: &str, address: &str, auth_b64: &str) -> Result<Vec<u8>> {
	let auth_key = help::get_session_key(tapp_id_hex, address).await;

	if auth_key.is_ok() && auth_b64 == auth_key.unwrap() {
		extend_auth(tapp_id_hex, address, auth_b64).await?;
		return help::result_ok();
	}

	None.ok_or_err("not_login")
}

pub async fn extend_auth(tapp_id_hex: &str, address: &str, auth_b64: &str) -> Result<()> {
	help::save_session_key(auth_b64.to_string(), tapp_id_hex, address).await
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogoutRequest {
	pub address: String,
}
pub fn txn_logout(_payload: Vec<u8>, _from_actor: String) -> Result<Vec<u8>> {
	// TODO
	help::result_ok()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpQueryBalanceRequest {
	pub tapp_id_b64: String,
	pub address: String,
	pub uuid: String,
	pub auth_b64: String,
	pub target: Option<String>,
	pub target_tapp_id_b64: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpQueryDepositRequest {
	pub tapp_id_b64: String,
	pub address: String,
	pub uuid: String,
	pub auth_b64: String,
	pub target: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryAssetRequest {
	pub tapp_id_b64: String,
	pub address: String,
	pub uuid: String,
	pub auth_b64: String,
	pub target: Option<String>,
}

pub async fn query_balance(payload: Vec<u8>, from_actor: String) -> Result<Vec<u8>> {
	let req: HttpQueryBalanceRequest = serde_json::from_slice(&payload)?;
	check_auth(&req.tapp_id_b64, &req.address, &req.auth_b64).await?;

	let query_account = match &req.target {
		Some(acct) => acct.to_string(),
		None => req.address.to_string(),
	};
	let query_token_id = match &req.target_tapp_id_b64 {
		Some(tid) => TokenId::from_hex(tid)?,
		None => TokenId::from_hex(&req.tapp_id_b64)?,
	};

	info!("begin to query tea balance => {:?}", query_account);

	let auth_key = base64::decode(&req.auth_b64)?;
	let uuid = req.uuid;
	let query_data = tappstore::TeaBalanceRequest {
		account: query_account,
		token_id: serialize(&query_token_id)?,
		auth_key,
	};

	request::send_tappstore_query(
		&from_actor,
		QueryTeaBalanceRequest(encode_protobuf(query_data)?),
		move |res| {
			Box::pin(async move {
				let r = tappstore::TeaBalanceResponse::decode(res.0.as_slice())?;
				let x = serde_json::json!({
					"balance": deserialize::<Balance,_>(&r.balance)?.to_string(),
					"ts": help::u128_from_le_buffer(&r.ts)?.to_string(),
					"uuid": uuid
				});

				help::cache_json_with_uuid(&uuid, x).await?;
				Ok(())
			})
		},
	)
	.await?;

	help::result_ok()
}

pub async fn query_deposit(payload: Vec<u8>, from_actor: String) -> Result<Vec<u8>> {
	let req: HttpQueryDepositRequest = serde_json::from_slice(&payload)?;
	check_auth(&req.tapp_id_b64, &req.address, &req.auth_b64).await?;

	info!("begin to query tea deposit");

	let auth_key = base64::decode(&req.auth_b64)?;
	let uuid = req.uuid;
	let query_data = tappstore::TeaBalanceRequest {
		account: req.address.to_string(),
		token_id: serialize(&TokenId::from_hex(&req.tapp_id_b64)?)?,
		auth_key,
	};

	request::send_tappstore_query(
		&from_actor,
		QueryTeaDepositRequest(encode_protobuf(query_data)?),
		move |res| {
			Box::pin(async move {
				let r = tappstore::TeaBalanceResponse::decode(res.0.as_slice())?;
				let x = serde_json::json!({
					"balance": deserialize::<Balance,_>(&r.balance)?.to_string(),
					"ts": help::u128_from_le_buffer(&r.ts)?.to_string(),
					"uuid": uuid
				});

				help::cache_json_with_uuid(&uuid, x).await?;
				Ok(())
			})
		},
	)
	.await?;

	help::result_ok()
}

pub async fn query_asset(payload: Vec<u8>, from_actor: String) -> Result<Vec<u8>> {
	let req: QueryAssetRequest = serde_json::from_slice(&payload)?;
	check_auth(&req.tapp_id_b64, &req.address, &req.auth_b64).await?;

	let query_account = match &req.target {
		Some(acct) => acct.to_string(),
		None => req.address.to_string(),
	};

	info!("begin to query asset => {:?}", query_account);

	let auth_key = base64::decode(&req.auth_b64)?;
	let uuid = req.uuid;
	let query_data = tappstore::AccountAssetRequest {
		account: query_account,
		token_id: serialize(&TokenId::from_hex(&req.tapp_id_b64)?)?,
		auth_key,
	};

	request::send_tappstore_query(
		&from_actor,
		FetchAccountAssetRequest(encode_protobuf(query_data)?),
		move |res| {
			Box::pin(async move {
				let r = tappstore::AccountAssetResponse::decode(res.0.as_slice())?;
				let x = serde_json::json!({
					"tea_balance": deserialize::<Balance,_>(&r.tea_balance)?.to_string(),
					"token_balance": deserialize::<Balance, _>(&r.token_balance)?.to_string(),
					"reserved_token_balance": deserialize::<Balance, _>(&r.reserved_token_balance)?.to_string(),
					"uuid": uuid,
				});
				help::cache_json_with_uuid(&uuid, x).await?;
				Ok(())
			})
		},
	)
	.await?;

	help::result_ok()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryAllowanceRequest {
	pub address: String,
	pub tapp_id_b64: String,
	pub uuid: String,
}
pub async fn query_allowance(payload: Vec<u8>, from_actor: String) -> Result<Vec<u8>> {
	let req: QueryAllowanceRequest = serde_json::from_slice(&payload)?;
	info!("query allowance... => {:?}", req);

	let uuid = req.uuid;
	let query_data = tappstore::TokenAllowanceRequest {
		account: req.address.to_string(),
		token_id: serialize(&TokenId::from_hex(&req.tapp_id_b64)?)?,
	};

	request::send_tappstore_query(
		&from_actor,
		FetchAllowanceRequest(encode_protobuf(query_data)?),
		move |res| {
			Box::pin(async move {
				let r = tappstore::TokenAllowanceResponse::decode(res.0.as_slice())?;
				let x = json!({
					"balance": deserialize::<Balance,_>(&r.balance)?.to_string(),
				});
				help::cache_json_with_uuid(&uuid, x).await?;
				Ok(())
			})
		},
	)
	.await?;

	help::result_ok()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryTappMetadataRequest {
	pub uuid: String,
	pub token_id: String,
}
pub async fn query_tapp_metadata(payload: Vec<u8>, from_actor: String) -> Result<Vec<u8>> {
	let req: QueryTappMetadataRequest = serde_json::from_slice(&payload)?;
	info!("query_tapp_metadata... => {:?}", req);

	let uuid = req.uuid;
	let query_data = tappstore::CommonSqlQueryRequest {
		msg: Some(
			tappstore::common_sql_query_request::Msg::QueryTappMetadataRequest(
				tappstore::QueryTappMetadataRequest {
					token_id: req.token_id,
				},
			),
		),
	};

	request::send_tappstore_query(
		&from_actor,
		CommonSqlQueryRequest(encode_protobuf(query_data)?),
		move |res| {
			Box::pin(async move {
				let r = tappstore::CommonSqlQueryResponse::decode(res.0.as_slice())?;
				let x = if !r.err.is_empty() {
					error!("query_tapp_metadata error: {}", &r.err);
					json!({
						"status": false,
						"error": &r.err,
					})
				} else {
					let data: tapp_common::tapp::TappMetaData = tea_codec::deserialize(&r.data)?;
					info!("query_tapp_metadata => {:?}", &data);

					json!({ "sql_query_result": data })
				};
				help::cache_json_with_uuid(&uuid, x).await?;
				Ok(())
			})
		},
	)
	.await?;

	help::result_ok()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryErrorLogRequest {
	pub uuid: String,
	pub query_type: String,
	pub query_key: String,
}
pub async fn query_error_log(_payload: Vec<u8>, _from_actor: String) -> Result<Vec<u8>> {
	todo!("do later");
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpQueryResultWithUuid {
	pub uuid: String,
}
pub async fn query_result(payload: Vec<u8>, from_actor: String) -> Result<Vec<u8>> {
	let req: HttpQueryResultWithUuid = serde_json::from_slice(&payload)?;
	match crate::query_cb::query_callback(from_actor, &req.uuid).await {
		Ok(res_val) => Ok(serde_json::to_vec(&res_val)?),
		Err(e) => help::result_error(e.to_string()),
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryHashRequest {
	pub uuid: String,
	pub hash: String,
	pub ts: String,
}
pub async fn query_txn_hash_result(payload: Vec<u8>, from_actor: String) -> Result<Vec<u8>> {
	let req: QueryHashRequest = serde_json::from_slice(&payload)?;
	info!("begin to query hash result...");

	let uuid = req.uuid;
	let txn_hash = hex::decode(req.hash.clone())?;
	let ts = tea_codec::serialize(&u128::from_str(&req.ts)?)?;

	let query_data = replica::FindExecutedTxnRequest { txn_hash, ts };

	request::send_tappstore_query(
		&from_actor,
		FindExecutedTxnRequest(encode_protobuf(query_data)?),
		move |res| {
			Box::pin(async move {
				let r = replica::FindExecutedTxnResponse::decode(res.0.as_slice())?;

				if r.success {
					if r.executed_txn.is_some() {
						info!("Txn hash return success. go to next step...");
						let x = json!({
							"status": true,
						});
						help::cache_json_with_uuid(&uuid, x).await?;
					} else {
						info!("Txn hash no error. but not success. wait for next loop...");

						let x = json!({
							"status": false,
							"error": "wait",
						});
						help::cache_json_with_uuid(&uuid, x).await?;
					}
				} else {
					let x = json!({
						"status": false,
						"error": &r.error_msg,
					});
					help::cache_json_with_uuid(&uuid, x).await?;
				}

				Ok(())
			})
		},
	)
	.await?;

	help::result_ok()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuerySystemVersionRequest {
	pub uuid: String,
}
pub async fn query_system_version(payload: Vec<u8>, from_actor: String) -> Result<Vec<u8>> {
	let req: QuerySystemVersionRequest = serde_json::from_slice(&payload)?;
	info!("query_system_version...");

	let uuid = req.uuid;

	request::send_tappstore_query(
		&from_actor,
		tappstore_actor_codec::QuerySystemVersionsRequest,
		move |res| {
			Box::pin(async move {
				let r = res.0;
				let x = json!({
					"client_version": r.client.version,
					"client_url": r.client.url,
					"enclave_version": r.enclave.version,
					"enclave_url": r.enclave.url,
				});
				help::cache_json_with_uuid(&uuid, x).await?;
				Ok(())
			})
		},
	)
	.await?;

	help::result_ok()
}
