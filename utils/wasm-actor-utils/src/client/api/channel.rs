#![allow(dead_code)]
#![allow(unused_imports)]

use prost::Message;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tea_runtime_codec::tapp::{Account, Balance, ChannelItem, ChannelItemStatus};
use tea_system_actors::payment_channel::{
	txns::PaymentChannelTxn, QueryChannelInfoRequest, QueryChannelInfoResponse, NAME,
};

use crate::client::{check_auth, help, request, Result};

const TARGET_ACTOR: &[u8] = NAME;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayerOpenChannelRequest {
	pub uuid: String,
	pub tapp_id_b64: String,
	pub address: String,
	pub auth_b64: String,

	pub channel_id: String,
	pub payee_address: String,
	pub grace_period: Option<u64>,
	pub fund_remaining: String,
	pub expire_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayerEarlyTerminateRequest {
	pub uuid: String,
	pub tapp_id_b64: String,
	pub address: String,
	pub auth_b64: String,

	pub channel_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayerTerminateRequest {
	pub uuid: String,
	pub tapp_id_b64: String,
	pub address: String,
	pub auth_b64: String,

	pub channel_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayerRefillRequest {
	pub uuid: String,
	pub tapp_id_b64: String,
	pub address: String,
	pub auth_b64: String,

	pub channel_id: String,
	pub refill_amount: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayeeUpdatePaymentRequest {
	pub uuid: String,
	pub tapp_id_b64: String,
	pub address: String,
	pub auth_b64: String,

	pub channel_id: String,
	pub sig: String,
	pub close_channel: bool,
	pub new_fund_remaining: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryChannelListWithAccountRequest {
	pub uuid: String,
	pub address: String,
	pub tapp_id_b64: String,
	pub auth_b64: String,
}

pub async fn open_payment_channel(payload: Vec<u8>, from_actor: String) -> Result<Vec<u8>> {
	let req: PayerOpenChannelRequest = serde_json::from_slice(&payload)?;
	check_auth(&req.tapp_id_b64, &req.address, &req.auth_b64).await?;

	info!("open_payment_channel ...");

	let cl = ChannelItem {
		channel_id: req.channel_id.parse()?,
		payer_address: req.address.parse()?,
		payee_address: req.payee_address.parse()?,
		fund_remaining: Balance::from_str_radix(&req.fund_remaining, 10)?,
		grace_period: match req.grace_period {
			Some(v) => Some(v),
			None => Some(3600_u64),
		},
		expire_time: Some(u128::from_str_radix(&req.expire_time, 10)?),
		status: ChannelItemStatus::Normal,
	};

	let txn = PaymentChannelTxn::OpenChannel {
		item: cl,
		auth_b64: req.auth_b64.clone(),
	};

	request::send_custom_txn(
		&from_actor,
		"open_payment_channel",
		&req.uuid,
		tea_codec::serialize(&req)?,
		tea_codec::serialize(&txn)?,
		vec![],
		TARGET_ACTOR,
	)
	.await?;

	help::result_ok()
}

pub async fn early_terminate(payload: Vec<u8>, from_actor: String) -> Result<Vec<u8>> {
	let req: PayerEarlyTerminateRequest = serde_json::from_slice(&payload)?;
	check_auth(&req.tapp_id_b64, &req.address, &req.auth_b64).await?;

	info!("early_terminate ...");

	let txn = PaymentChannelTxn::PayerEarlyTerminate {
		channel_id: req.channel_id.parse()?,
		auth_b64: req.auth_b64.clone(),
	};

	request::send_custom_txn(
		&from_actor,
		"early_terminate",
		&req.uuid,
		tea_codec::serialize(&req)?,
		tea_codec::serialize(&txn)?,
		vec![],
		TARGET_ACTOR,
	)
	.await?;

	help::result_ok()
}

pub async fn terminate(payload: Vec<u8>, from_actor: String) -> Result<Vec<u8>> {
	let req: PayerTerminateRequest = serde_json::from_slice(&payload)?;
	check_auth(&req.tapp_id_b64, &req.address, &req.auth_b64).await?;

	info!("terminate ...");

	let txn = PaymentChannelTxn::PayerTerminate {
		channel_id: req.channel_id.parse()?,
		auth_b64: req.auth_b64.clone(),
	};

	request::send_custom_txn(
		&from_actor,
		"terminate",
		&req.uuid,
		tea_codec::serialize(&req)?,
		tea_codec::serialize(&txn)?,
		vec![],
		TARGET_ACTOR,
	)
	.await?;

	help::result_ok()
}

pub async fn refill_fund(payload: Vec<u8>, from_actor: String) -> Result<Vec<u8>> {
	let req: PayerRefillRequest = serde_json::from_slice(&payload)?;
	check_auth(&req.tapp_id_b64, &req.address, &req.auth_b64).await?;

	info!("refill_fund ...");

	let txn = PaymentChannelTxn::PayerRefill {
		channel_id: req.channel_id.parse()?,
		auth_b64: req.auth_b64.clone(),
		refill_amount: Balance::from_str_radix(&req.refill_amount, 10)?,
	};

	request::send_custom_txn(
		&from_actor,
		"refill_fund",
		&req.uuid,
		tea_codec::serialize(&req)?,
		tea_codec::serialize(&txn)?,
		vec![],
		TARGET_ACTOR,
	)
	.await?;

	help::result_ok()
}

pub async fn query_channel_list_with_account(
	payload: Vec<u8>,
	from_actor: String,
) -> Result<Vec<u8>> {
	let req: QueryChannelListWithAccountRequest = serde_json::from_slice(&payload)?;
	check_auth(&req.tapp_id_b64, &req.address, &req.auth_b64).await?;

	info!("query_channel_list_with_account ...");

	let query_data: QueryChannelInfoRequest = QueryChannelInfoRequest(req.address.parse()?);

	let uuid = req.uuid;
	let res: QueryChannelInfoResponse =
		request::send_custom_query(&from_actor, query_data, TARGET_ACTOR).await?;

	let x = serde_json::json!({
		"payer_list": res.payer_list,
		"payee_list": res.payee_list,
	});
	info!("query query_channel_list_with_account => {:?}", x);

	help::cache_json_with_uuid(&uuid, x).await?;

	help::result_ok()
}

pub async fn payee_update_payment(payload: Vec<u8>, from_actor: String) -> Result<Vec<u8>> {
	let req: PayeeUpdatePaymentRequest = serde_json::from_slice(&payload)?;
	check_auth(&req.tapp_id_b64, &req.address, &req.auth_b64).await?;

	info!("payee_update_payment ...");

	let txn = PaymentChannelTxn::UpdatePayment {
		channel_id: req.channel_id.parse()?,
		auth_b64: req.auth_b64.clone(),
		payment_update_sig: req.sig.to_string(),
		new_fund_remaining: Balance::from_str_radix(&req.new_fund_remaining, 10)?,
		close_channel: req.close_channel,
	};

	request::send_custom_txn(
		&from_actor,
		"payee_update_payment",
		&req.uuid,
		tea_codec::serialize(&req)?,
		tea_codec::serialize(&txn)?,
		vec![],
		TARGET_ACTOR,
	)
	.await?;

	help::result_ok()
}
