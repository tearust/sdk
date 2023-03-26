use crate::enclave::actors::env::tappstore_id;
use crate::enclave::error::{Actor, Errors, GlueSqlErrors, Result};
use gluesql_core::prelude::{Payload, Row, Value};
use prost::Message;
use tea_actorx_core::RegId;
use tea_actorx_runtime::call;
use tea_codec::{deserialize, serialize, ResultExt};
use tea_runtime_codec::actor_txns::context::TokenContext;
use tea_runtime_codec::tapp::{Account, Balance, TokenId};
use tea_runtime_codec::vmh::message::{encode_protobuf, structs_proto::tokenstate};
use tea_system_actors::tokenstate::*;

pub async fn get_magic_number() -> Result<u64> {
	let n = call(RegId::Static(NAME).inst(0), GetMagicNumberRequest).await?;
	Ok(n.0)
}

pub async fn is_in_sql_transaction(token_id: TokenId) -> Result<bool> {
	let buf = call(
		RegId::Static(NAME).inst(0),
		SqlIsInTransactionRequest(encode_protobuf(tokenstate::IsInTransactionRequest {
			token_id: serialize(&token_id)?,
		})?),
	)
	.await?;
	let res = tokenstate::IsInTransactionResponse::decode(buf.0.as_slice())?;
	Ok(res.yes)
}

pub async fn cancel_sql_transaction(token_id: TokenId) -> Result<()> {
	call(
		RegId::Static(NAME).inst(0),
		SqlCancelTransactionRequest(encode_protobuf(tokenstate::CancelTransactionRequest {
			token_id: serialize(&token_id)?,
		})?),
	)
	.await?;
	Ok(())
}

pub async fn table_row_count(token_id: TokenId, table_name: &str) -> Result<u64> {
	let result = sql_query(token_id, format!(r"SELECT COUNT(*) FROM {table_name};")).await?;
	query_first_u64(
		result
			.first()
			.ok_or_else(|| GlueSqlErrors::InvalidTableRowCount(table_name.to_string(), token_id))?,
	)
}

pub fn query_select_rows(payload: &Payload) -> Result<&Vec<Row>> {
	match payload {
		Payload::Select { labels: _, rows } => Ok(rows),
		_ => Err(GlueSqlErrors::InvalidSelectResult.into()),
	}
}

pub fn query_first_row(payload: &Payload) -> Result<&Row> {
	let rows = query_select_rows(payload)?;
	rows.get(0).ok_or(GlueSqlErrors::InvalidFirstRow.into())
}

pub fn query_all_first_columns_as_u64(payload: &Payload) -> Result<Vec<u64>> {
	let mut rtn = Vec::new();
	for v in query_select_rows(payload)? {
		if let Some(cml_id) = v.get_value_by_index(0) {
			rtn.push(sql_value_to_u64(cml_id)?);
		}
	}
	Ok(rtn)
}

pub fn query_all_first_columns_as_string(payload: &Payload) -> Result<Vec<&str>> {
	let mut rtn = Vec::new();
	for v in query_select_rows(payload)? {
		if let Some(cml_id) = v.get_value_by_index(0) {
			rtn.push(sql_value_to_string(cml_id)?);
		}
	}
	Ok(rtn)
}

pub fn query_first_u64(payload: &Payload) -> Result<u64> {
	let row = query_first_row(payload)?;
	let count = row
		.get_value_by_index(0)
		.ok_or(GlueSqlErrors::InvalidFirstValue)?;
	sql_value_to_u64(count)
}

pub fn query_first_string(payload: &Payload) -> Result<&str> {
	let row = query_first_row(payload)?;
	let value = row
		.get_value_by_index(0)
		.ok_or(GlueSqlErrors::InvalidFirstValue)?;
	sql_value_to_string(value)
}

pub fn sql_value_to_u64(value: &Value) -> Result<u64> {
	match value {
		Value::I64(i) => Ok(*i as u64),
		_ => Err(GlueSqlErrors::InvalidI64(format!("{value:?}")).into()),
	}
}

pub fn sql_value_to_option_u64(value: &Value) -> Result<Option<u64>> {
	match value {
		Value::Null => Ok(None),
		_ => Ok(Some(sql_value_to_u64(value)?)),
	}
}

pub fn sql_value_to_string(value: &Value) -> Result<&str> {
	match value {
		Value::Str(s) => Ok(s),
		_ => Err(GlueSqlErrors::InvalidString(format!("{value:?}")).into()),
	}
}

pub fn sql_value_to_option_string(value: &Value) -> Result<Option<&str>> {
	match value {
		Value::Null => Ok(None),
		_ => Ok(Some(sql_value_to_string(value)?)),
	}
}

pub async fn sql_query(token_id: TokenId, sql: String) -> Result<Vec<Payload>> {
	let req = tokenstate::ExecGlueQueryRequest {
		token_id: serialize(&token_id)?,
		sql,
	};

	let res = call(
		RegId::Static(NAME).inst(0),
		ExecGlueQueryRequest(encode_protobuf(req)?),
	)
	.await?;
	let res = tokenstate::ExecGlueQueryResponse::decode(res.0.as_slice())?;
	res.payloads
		.iter()
		.map(|buf| deserialize::<Payload, _>(buf.as_slice()))
		.collect::<Result<_, _>>()
		.err_into()
}

pub async fn sql_query_first(token_id: TokenId, sql: String) -> Result<Payload> {
	let mut payloads = sql_query(token_id, sql.clone()).await?;
	if payloads.is_empty() {
		return Err(GlueSqlErrors::InvalidFirstPayload(sql, token_id).into());
	}
	Ok(payloads.remove(0))
}

pub async fn mov(from: Account, to: Account, amt: Balance, ctx: Vec<u8>) -> Result<Vec<u8>> {
	if amt.is_zero() {
		info!("Mov 0 unit, ignored.");
		return Ok(ctx);
	}

	let res: MoveResponse = call::<_, Actor>(
		RegId::Static(NAME).inst(0),
		MoveRequest { from, to, amt, ctx },
	)
	.await
	.map_err(|e| Errors::StateMachineMoveFailed(from.to_string(), to.to_string(), amt, e.into()))?;
	Ok(res.0)
}

/// move TEA across token. That means from one token_id balance to another token_id balance
/// Most likely move from TAppStore balance to a TAppToken balance due to buy/sell token
pub async fn cross_move(
	from: Account,
	to: Account,
	amt: Balance,            //unit is TEA
	from_ctx_bytes: Vec<u8>, //
	to_ctx_bytes: Vec<u8>,   //still TEA
) -> Result<(Vec<u8>, Vec<u8>)> {
	if amt == 0.into() {
		info!("Cross move 0 unit, ignored.");
		return Ok((from_ctx_bytes, to_ctx_bytes));
	}

	let from_ctx: TokenContext = deserialize(&from_ctx_bytes)?;
	let to_ctx: TokenContext = deserialize(&to_ctx_bytes)?;

	if from_ctx.tid == to_ctx.tid {
		// same token id, move.
		let from_ctx_bytes = mov(from, to, amt, from_ctx_bytes).await?;
		return Ok((from_ctx_bytes, to_ctx_bytes));
	}

	let res = call::<_, Actor>(
		RegId::Static(NAME).inst(0),
		CrossMoveRequest {
			from,
			to,
			amt,
			from_ctx: from_ctx_bytes,
			to_ctx: to_ctx_bytes,
		},
	)
	.await
	.map_err(|e| {
		Errors::StateMachineCrossMoveFailed(
			from_ctx.tid.to_hex(),
			from.to_string(),
			to_ctx.tid.to_hex(),
			to.to_string(),
			amt,
			e.into(),
		)
	})?;
	Ok((res.from_ctx, res.to_ctx))
}

pub async fn api_cross_move(
	from: Account,
	to: Account,
	amt: Balance,            //unit is TEA
	from_ctx_bytes: Vec<u8>, //
	to_ctx_bytes: Vec<u8>,   //still TEA
) -> Result<(Vec<u8>, Vec<u8>)> {
	if amt == 0.into() {
		info!("Api Cross move 0 unit, ignored.");
		return Ok((from_ctx_bytes, to_ctx_bytes));
	}

	let from_ctx: TokenContext = deserialize(&from_ctx_bytes)?;
	let to_ctx: TokenContext = deserialize(&to_ctx_bytes)?;

	if from_ctx.tid == to_ctx.tid {
		// same token id, move.
		let from_ctx_bytes = mov(from, to, amt, from_ctx_bytes).await?;
		return Ok((from_ctx_bytes, to_ctx_bytes));
	}

	let res = call::<_, Actor>(
		RegId::Static(NAME).inst(0),
		ApiCrossMoveRequest {
			from,
			to,
			amt,
			from_ctx: from_ctx_bytes,
			to_ctx: to_ctx_bytes,
			tappstore_id: tappstore_id().await?,
		},
	)
	.await
	.map_err(|e| {
		Errors::StateMachineCrossMoveFailed(
			from_ctx.tid.to_hex(),
			from.to_string(),
			to_ctx.tid.to_hex(),
			to.to_string(),
			amt,
			e.into(),
		)
	})?;
	Ok((res.from_ctx, res.to_ctx))
}

pub async fn api_deposit(acct: Account, amt: Balance, ctx: Vec<u8>) -> Result<Vec<u8>> {
	let buf = encode_protobuf(tokenstate::DepositRequest {
		acct: serialize(&acct)?,
		amt: serialize(&amt)?,
		ctx,
	})?;
	let res_buf = call(
		RegId::Static(codec::NAME).inst(0),
		codec::ApiDepositRequest(buf),
	)
	.await?;
	let res = StateOperateResponse::decode(res_buf.0.as_slice())?;
	let operate_error: Error = deserialize(&res.operate_error)?;
	if operate_error.summary().as_deref() == Some(OPERATE_ERROR_SUCCESS_SUMMARY) {
		info!("actor_statemachine api_deposit successed");
		Ok(res.ctx)
	} else {
		error!(
			"actor_statemachine api_deposit error {}",
			operate_error.to_string()
		);
		Err(operate_error.into())
	}
}

pub async fn api_refund(acct: Account, amt: Balance, ctx: Vec<u8>) -> Result<Vec<u8>> {
	let buf = encode_protobuf(tokenstate::RefundRequest {
		acct: serialize(&acct)?,
		amt: serialize(&amt)?,
		ctx,
	})?;
	let res_buf = call(
		RegId::Static(codec::NAME).inst(0),
		codec::ApiRefundRequest(buf),
	)
	.await?;
	let res = StateOperateResponse::decode(res_buf.0.as_slice())?;
	let operate_error: Error = deserialize(&res.operate_error)?;
	if operate_error.summary().as_deref() == Some(OPERATE_ERROR_SUCCESS_SUMMARY) {
		info!("actor_statemachine api_refund successed");
		Ok(res.ctx)
	} else {
		error!(
			"actor_statemachine api_refund error {}",
			operate_error.to_string()
		);
		Err(operate_error.into())
	}
}
