use crate::error::{GlueSqlErrors, Result};
use gluesql_core::prelude::{Payload, Value};
use prost::Message;
use tapp_common::TokenId;
use tea_actorx_core::RegId;
use tea_actorx_runtime::call;
use tea_codec::{deserialize, serialize, ResultExt};
use tokenstate_actor_codec::*;
use vmh_codec::message::{encode_protobuf, structs_proto::tokenstate};

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

pub fn query_select_rows(payload: &Payload) -> Result<&Vec<Vec<Value>>> {
    match payload {
        Payload::Select { labels: _, rows } => Ok(rows),
        _ => Err(GlueSqlErrors::InvalidSelectResult.into()),
    }
}

pub fn query_first_row(payload: &Payload) -> Result<&Vec<Value>> {
    let rows = query_select_rows(payload)?;
    rows.get(0).ok_or(GlueSqlErrors::InvalidFirstRow.into())
}

pub fn query_all_first_columns_as_u64(payload: &Payload) -> Result<Vec<u64>> {
    let mut rtn = Vec::new();
    for v in query_select_rows(payload)? {
        if let Some(cml_id) = v.get(0) {
            rtn.push(sql_value_to_u64(cml_id)?);
        }
    }
    Ok(rtn)
}

pub fn query_all_first_columns_as_string(payload: &Payload) -> Result<Vec<&str>> {
    let mut rtn = Vec::new();
    for v in query_select_rows(payload)? {
        if let Some(cml_id) = v.get(0) {
            rtn.push(sql_value_to_string(cml_id)?);
        }
    }
    Ok(rtn)
}

pub fn query_first_u64(payload: &Payload) -> Result<u64> {
    let row = query_first_row(payload)?;
    let count = row.get(0).ok_or(GlueSqlErrors::InvalidFirstValue)?;
    sql_value_to_u64(count)
}

pub fn query_first_string(payload: &Payload) -> Result<&str> {
    let row = query_first_row(payload)?;
    let value = row.get(0).ok_or(GlueSqlErrors::InvalidFirstValue)?;
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
