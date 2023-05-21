use crate::enclave::actors::env::tappstore_id;
use crate::enclave::error::{Error, Errors, GlueSqlErrors, Result};
use gluesql_core::prelude::{Payload, Row, Value};
use prost::Message;
use tea_actorx::ActorId;
use tea_codec::{deserialize, serialize, ResultExt};
use tea_runtime_codec::actor_txns::context::TokenContext;
use tea_runtime_codec::tapp::{Account, Balance, TokenId};
use tea_runtime_codec::vmh::message::{encode_protobuf, structs_proto::tokenstate};
use tea_system_actors::tokenstate::*;

const OPERATE_ERROR_SUCCESS_SUMMARY: &str = "OP_101__success_without_context_change";

/// Return magic number from settings.
/// The op-code is GetMagicNumberRequest.
pub async fn get_magic_number() -> Result<u64> {
	let n = ActorId::Static(NAME).call(GetMagicNumberRequest).await?;
	Ok(n.0)
}

/// TODO
pub async fn is_in_sql_transaction(token_id: TokenId) -> Result<bool> {
	let buf = ActorId::Static(NAME)
		.call(SqlIsInTransactionRequest(encode_protobuf(
			tokenstate::IsInTransactionRequest {
				token_id: serialize(&token_id)?,
			},
		)?))
		.await?;
	let res = tokenstate::IsInTransactionResponse::decode(buf.0.as_slice())?;
	Ok(res.yes)
}

/// TODO
pub async fn cancel_sql_transaction(token_id: TokenId) -> Result<()> {
	ActorId::Static(NAME)
		.call(SqlCancelTransactionRequest(encode_protobuf(
			tokenstate::CancelTransactionRequest {
				token_id: serialize(&token_id)?,
			},
		)?))
		.await?;
	Ok(())
}

/// Return total count of table with token id in sql.
pub async fn table_row_count(token_id: TokenId, table_name: &str) -> Result<u64> {
	let result = sql_query(token_id, format!(r"SELECT COUNT(*) FROM {table_name};")).await?;
	query_first_u64(
		result
			.first()
			.ok_or_else(|| GlueSqlErrors::InvalidTableRowCount(table_name.to_string(), token_id))?,
	)
}

/// Return all select rows of payload.
/// 	let list_sql = sql_query_first(
/// 		tappstore_id().await?,
/// 		"SELECT * FROM TxnGasFeeTable".to_string(),
/// 	)
/// 	.await?;
/// 	let list = query_select_rows(&list_sql)?;
pub fn query_select_rows(payload: &Payload) -> Result<&Vec<Row>> {
	match payload {
		Payload::Select { labels: _, rows } => Ok(rows),
		_ => Err(GlueSqlErrors::InvalidSelectResult.into()),
	}
}

/// Return the first row of payload.
pub fn query_first_row(payload: &Payload) -> Result<&Row> {
	let rows = query_select_rows(payload)?;
	rows.get(0).ok_or(GlueSqlErrors::InvalidFirstRow.into())
}

/// Return all select data of payload and transform to u64.
pub fn query_all_first_columns_as_u64(payload: &Payload) -> Result<Vec<u64>> {
	let mut rtn = Vec::new();
	for v in query_select_rows(payload)? {
		if let Some(cml_id) = v.get_value_by_index(0) {
			rtn.push(sql_value_to_u64(cml_id)?);
		}
	}
	Ok(rtn)
}

/// Return all select data of payload and transform to string.
pub fn query_all_first_columns_as_string(payload: &Payload) -> Result<Vec<&str>> {
	let mut rtn = Vec::new();
	for v in query_select_rows(payload)? {
		if let Some(cml_id) = v.get_value_by_index(0) {
			rtn.push(sql_value_to_string(cml_id)?);
		}
	}
	Ok(rtn)
}

/// Return first select data of payload and transform to u64.
pub fn query_first_u64(payload: &Payload) -> Result<u64> {
	let row = query_first_row(payload)?;
	let count = row
		.get_value_by_index(0)
		.ok_or(GlueSqlErrors::InvalidFirstValue)?;
	sql_value_to_u64(count)
}

/// Return first select data of payload and transform to string.
pub fn query_first_string(payload: &Payload) -> Result<&str> {
	let row = query_first_row(payload)?;
	let value = row
		.get_value_by_index(0)
		.ok_or(GlueSqlErrors::InvalidFirstValue)?;
	sql_value_to_string(value)
}

/// Transform a sql value to u64.
pub fn sql_value_to_u64(value: &Value) -> Result<u64> {
	match value {
		Value::I64(i) => Ok(*i as u64),
		_ => Err(GlueSqlErrors::InvalidI64(format!("{value:?}")).into()),
	}
}

/// Transform a sql value to an option u64.
/// It used to transform a sql value can be NULL.
pub fn sql_value_to_option_u64(value: &Value) -> Result<Option<u64>> {
	match value {
		Value::Null => Ok(None),
		_ => Ok(Some(sql_value_to_u64(value)?)),
	}
}

/// Transform a sql value to string.
pub fn sql_value_to_string(value: &Value) -> Result<&str> {
	match value {
		Value::Str(s) => Ok(s),
		_ => Err(GlueSqlErrors::InvalidString(format!("{value:?}")).into()),
	}
}

/// Transform a sql value to an option string.
/// It used to transform a sql value can be NULL.
pub fn sql_value_to_option_string(value: &Value) -> Result<Option<&str>> {
	match value {
		Value::Null => Ok(None),
		_ => Ok(Some(sql_value_to_string(value)?)),
	}
}

/// Generate a vec sql payload.
pub async fn sql_query(token_id: TokenId, sql: String) -> Result<Vec<Payload>> {
	let req = tokenstate::ExecGlueQueryRequest {
		token_id: serialize(&token_id)?,
		sql,
	};

	let res = ActorId::Static(NAME)
		.call(ExecGlueQueryRequest(encode_protobuf(req)?))
		.await?;
	let res = tokenstate::ExecGlueQueryResponse::decode(res.0.as_slice())?;
	res.payloads
		.iter()
		.map(|buf| deserialize::<Payload, _>(buf.as_slice()))
		.collect::<Result<_, _>>()
		.err_into()
}

/// Generate sql payload.
///  	let list_sql = sql_query_first(
/// 		tappstore_id().await?,
/// 		"SELECT * FROM TxnGasFeeTable".to_string(),
/// 	)
pub async fn sql_query_first(token_id: TokenId, sql: String) -> Result<Payload> {
	let mut payloads = sql_query(token_id, sql.clone()).await?;
	if payloads.is_empty() {
		return Err(GlueSqlErrors::InvalidFirstPayload(sql, token_id).into());
	}
	Ok(payloads.remove(0))
}

/// Move token from user to another user with same token id.
/// It usually used to move TEA under GLOBAL token_id, which is tappstore in system.
pub async fn mov(from: Account, to: Account, amt: Balance, ctx: Vec<u8>) -> Result<Vec<u8>> {
	if amt.is_zero() {
		info!("Mov 0 unit, ignored.");
		return Ok(ctx);
	}

	let res: MoveResponse = ActorId::Static(NAME)
		.call(MoveRequest { from, to, amt, ctx })
		.await
		.map_err(|e| {
			Errors::StateMachineMoveFailed(from.to_string(), to.to_string(), amt, e.into())
		})?;
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

	let res = ActorId::Static(NAME)
		.call(CrossMoveRequest {
			from,
			to,
			amt,
			from_ctx: from_ctx_bytes,
			to_ctx: to_ctx_bytes,
		})
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

/// An warpped method for cross_move, which include the allowance opreation.
/// Developer can easily to move TEA in different token_id and doesnot need to operate the user allowance manually.
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

	let res = ActorId::Static(NAME)
		.call(ApiCrossMoveRequest {
			from,
			to,
			amt,
			from_ctx: from_ctx_bytes,
			to_ctx: to_ctx_bytes,
			tappstore_id: tappstore_id().await?,
		})
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

/// An warpped method for deposit, which include the allowance opreation.
pub async fn api_deposit(
	acct: Account,
	amt: Balance,
	tappstore_ctx: Vec<u8>,
	token_ctx: Vec<u8>,
) -> Result<(Vec<u8>, Vec<u8>)> {
	let buf = encode_protobuf(tokenstate::ApiDepositRequest {
		acct: serialize(&acct)?,
		amt: serialize(&amt)?,
		ctx: tappstore_ctx,
		token_ctx,
	})?;
	let res_buf = ActorId::Static(NAME).call(ApiDepositRequest(buf)).await?;
	let res = tokenstate::ApiStateOperateResponse::decode(res_buf.0.as_slice())?;
	let operate_error: Error = deserialize(&res.operate_error)?;
	if operate_error.summary().as_deref() == Some(OPERATE_ERROR_SUCCESS_SUMMARY) {
		info!("actor_statemachine api_deposit successed");
		Ok((res.ctx, res.token_ctx))
	} else {
		error!(
			"actor_statemachine api_deposit error {}",
			operate_error.to_string()
		);
		Err(operate_error)
	}
}

/// An warpped method for refund, which include the allowance opreation.
pub async fn api_refund(
	acct: Account,
	amt: Balance,
	tappstore_ctx: Vec<u8>,
	token_ctx: Vec<u8>,
) -> Result<(Vec<u8>, Vec<u8>)> {
	let buf = encode_protobuf(tokenstate::ApiRefundRequest {
		acct: serialize(&acct)?,
		amt: serialize(&amt)?,
		ctx: tappstore_ctx,
		token_ctx,
	})?;
	let res_buf = ActorId::Static(NAME).call(ApiRefundRequest(buf)).await?;
	let res = tokenstate::ApiStateOperateResponse::decode(res_buf.0.as_slice())?;
	let operate_error: Error = deserialize(&res.operate_error)?;
	if operate_error.summary().as_deref() == Some(OPERATE_ERROR_SUCCESS_SUMMARY) {
		info!("actor_statemachine api_refund successed");
		Ok((res.ctx, res.token_ctx))
	} else {
		error!(
			"actor_statemachine api_refund error {}",
			operate_error.to_string()
		);
		Err(operate_error)
	}
}
