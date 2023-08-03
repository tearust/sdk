use super::enclave::random_u64;
use crate::enclave::{
	actors::{
		env::tappstore_id, persist::async_persist_request, replica::send_transaction_locally,
	},
	error::{Error, Errors, Result},
};
use prost::Message;
use tea_actorx_core::RegId;
use tea_actorx_runtime::call;
use tea_codec::{deserialize, serialize, ResultExt};
use tea_runtime_codec::tapp::{
	statement::TypedStatement, Account, AuthKey, Balance, TokenId, GOD_MODE_AUTH_KEY,
};
use tea_runtime_codec::vmh::message::{
	encode_protobuf,
	structs_proto::{persist, tokenstate::*},
};
use tea_runtime_codec::{
	actor_txns::{
		auth::TokenAuthOp,
		context::{ReadConflictMode, TokenContext},
		tsid::Tsid,
		TxnSerial,
	},
	tapp::RECEIPTING_AUTH_KEY,
};
use tea_system_actors::tappstore::txns::TappstoreTxn;
use tea_system_actors::tokenstate::{self as codec};

#[derive(Debug, Default, Clone)]
pub struct CommitContext {
	pub ctx: Vec<u8>,
	pub gluedb_ctx: Option<GluedbTransactionContext>,
	pub payee_miner_ctx: Option<Vec<u8>>,
	pub payee_app_ctx: Option<Vec<u8>>,
	pub auth_key: AuthKey,
	pub memo: String,
}

impl CommitContext {
	pub fn new(
		ctx: Vec<u8>,
		gluedb_ctx: Option<GluedbTransactionContext>,
		payee_miner_ctx: Option<Vec<u8>>,
		payee_app_ctx: Option<Vec<u8>>,
		auth_key: AuthKey,
		memo: String,
	) -> CommitContext {
		CommitContext {
			ctx,
			gluedb_ctx,
			payee_miner_ctx,
			payee_app_ctx,
			auth_key,
			memo,
		}
	}

	pub fn ctx_god_mode(ctx: Vec<u8>) -> CommitContext {
		CommitContext {
			ctx,
			auth_key: GOD_MODE_AUTH_KEY,
			..Default::default()
		}
	}

	pub fn ctx_receipting(ctx: Vec<u8>, memo: String) -> CommitContext {
		CommitContext {
			ctx,
			memo,
			auth_key: RECEIPTING_AUTH_KEY,
			..Default::default()
		}
	}

	pub fn log_from_bytes(&self) -> Result<String> {
		let mut str = String::new();
		str.push_str(&format!(
			"\nTAppStoreContext:\n\t{}\n\t{}\n\t{}\n\t{}",
			TokenContext::log_tea_from_bytes(&self.ctx)?,
			TokenContext::log_deposit_from_bytes(&self.ctx)?,
			TokenContext::log_bonding_from_bytes(&self.ctx)?,
			TokenContext::log_allowance_from_bytes(&self.ctx)?,
		));
		if let Some(payee_miner_ctx) = &self.payee_miner_ctx {
			str.push_str(&format!(
				"\nPayeeMinerContext:\n\t{}\n\t{}\n\t{}\n\t{}",
				TokenContext::log_tea_from_bytes(payee_miner_ctx)?,
				TokenContext::log_deposit_from_bytes(payee_miner_ctx)?,
				TokenContext::log_bonding_from_bytes(payee_miner_ctx)?,
				TokenContext::log_allowance_from_bytes(payee_miner_ctx)?,
			));
		}
		if let Some(payee_app_ctx) = &self.payee_app_ctx {
			str.push_str(&format!(
				"\nPayeeAppContext:\n\t{}\n\t{}\n\t{}\n\t{}",
				TokenContext::log_tea_from_bytes(payee_app_ctx)?,
				TokenContext::log_deposit_from_bytes(payee_app_ctx)?,
				TokenContext::log_bonding_from_bytes(payee_app_ctx)?,
				TokenContext::log_allowance_from_bytes(payee_app_ctx)?,
			));
		}
		Ok(str)
	}
}

#[derive(Debug, Default, Clone)]
pub struct CommitContextList {
	pub ctx_list: Vec<CommitContext>,
	pub neutralize_hidden_acct_credit: Balance,
	pub neutralize_hidden_acct_debit: Balance,
}

impl CommitContextList {
	pub async fn check(&self) -> Result<()> {
		for ctx in self.ctx_list.iter() {
			check(ctx.clone()).await?;
		}
		Ok(())
	}

	pub async fn commit(&self, base: Tsid, tsid: Tsid) -> Result<()> {
		// check all contexts to avoid errors during actual commit
		self.check().await?;

		if self.ctx_list.is_empty() {
			//what if the ctx_list is not empty but not including tappstore_ctx?
			info!("commit context list is empty, commit placeholder context instead.");
			let placeholder_ctx =
				serialize(&TokenContext::new_slim(tsid, base, tappstore_id().await?))?;
			let (credit, debit, _) = commit(CommitContext::ctx_god_mode(placeholder_ctx)).await?;
			assert_eq!(credit, Balance::zero());
			assert_eq!(debit, Balance::zero());
			return Ok(());
		}

		let mut global_statements = vec![];
		let mut actual_neutral_balance: (Balance, Balance) = (Balance::zero(), Balance::zero());
		for ctx in self.ctx_list.iter() {
			let (hidden_acct_credit, hidden_acct_debit, statements) = commit(ctx.clone()).await?;
			actual_neutral_balance = (
				actual_neutral_balance.0 + hidden_acct_credit,
				actual_neutral_balance.1 + hidden_acct_debit,
			);

			if !statements.is_empty() {
				let timestamp = self.try_get_ctx_timestamp(&ctx.ctx)?;
				global_statements.push((statements, timestamp, ctx.memo.clone()));
			}
		}

		info!(
			"actual_neutual_balance: {:?}, neutual_hidden_acct_balance: (credit: {}, debit: {})",
			&actual_neutral_balance,
			self.neutralize_hidden_acct_credit,
			self.neutralize_hidden_acct_debit,
		);

		if (actual_neutral_balance.0 != self.neutralize_hidden_acct_debit
			|| actual_neutral_balance.1 != self.neutralize_hidden_acct_credit)
			&& actual_neutral_balance.0 != actual_neutral_balance.1
		{
			error!(
				"******** Commit successfully, but Neutralize the hidden system account failed.\
				 The total TEA is no longer balance in layer two"
			);
			return Err(Errors::NeutralizeExpectation(
				self.neutralize_hidden_acct_credit,
				self.neutralize_hidden_acct_debit,
				actual_neutral_balance,
			)
			.into());
		}

		if !global_statements.is_empty() {
			let res = async_persist_request(persist::PersistRequest {
				msg: Some(persist::persist_request::Msg::AppendStatements(
					persist::AppendStatements {
						statements_serial: serialize(&global_statements)?,
					},
				)),
				..Default::default()
			})
			.await?;
			if let Some(persist::persist_response::Msg::ErrorMessage(e)) = res.msg.as_ref() {
				warn!(
					"persist statements with tsid {:?} failed: {}",
					tsid, e.message
				);
			}
		}

		Ok(())
	}

	fn try_get_ctx_timestamp(&self, ctx: &[u8]) -> Result<u128> {
		let ctx: TokenContext = deserialize(ctx)?;
		Ok(ctx.tsid.ts)
	}
	pub fn log_from_bytes(&self) -> Result<String> {
		let mut str = String::new();
		for c in self.ctx_list.iter() {
			str.push_str(&c.log_from_bytes()?);
		}
		Ok(str)
	}
}

impl TryInto<CommitRequest> for CommitContext {
	type Error = Error;

	fn try_into(self) -> Result<CommitRequest, Self::Error> {
		Ok(CommitRequest {
			ctx: self.ctx,
			gluedb_ctx: self.gluedb_ctx,
			auth_key: serialize(&self.auth_key)?,
			payee_miner_ctx: self.payee_miner_ctx,
			payee_app_ctx: self.payee_app_ctx,
		})
	}
}

pub async fn query_state_tsid() -> Result<Tsid> {
	let buf = call(
		RegId::Static(codec::NAME).inst(0),
		codec::QueryStateTsidRequest,
	)
	.await?;
	let res = QueryStateTsidResponse::decode(buf.0.as_slice())?;
	let tsid: Tsid = deserialize(res.state_tsid)?;
	Ok(tsid)
}

pub async fn check(ctx: CommitContext) -> Result<()> {
	let buf = encode_protobuf::<CommitRequest>(ctx.try_into()?)?;
	call(
		RegId::Static(codec::NAME).inst(0),
		codec::CheckTxnRequest(buf),
	)
	.await?;
	Ok(())
}

/// return value (hidden_acct_credit, hidden_acct_debit)
/// we will need those two value to check the overall balance
/// after commit.
/// we need to make sure the overall balance is zero
pub async fn commit(ctx: CommitContext) -> Result<(Balance, Balance, Vec<TypedStatement>)> {
	let buf = encode_protobuf::<CommitRequest>(ctx.try_into()?)?;
	let res_buf = call(
		RegId::Static(codec::NAME).inst(0),
		codec::CommitTxnRequest(buf),
	)
	.await?;
	let res = StateCommitResponse::decode(res_buf.0.as_slice())?;
	let hidden_acct_credit: Balance = deserialize(&res.hidden_acct_credit)?;
	let hidden_acct_debit: Balance = deserialize(&res.hidden_acct_debit)?;
	let statements: Vec<TypedStatement> = deserialize(&res.statements_bytes)?;
	Ok((hidden_acct_credit, hidden_acct_debit, statements))
}

pub async fn new_txn_serial(
	actor_name: &[u8],
	bytes: Vec<u8>,
	gas_limit: u64,
) -> Result<TxnSerial> {
	let nonce = random_u64().await?;
	Ok(TxnSerial::new(actor_name.to_vec(), bytes, nonce, gas_limit))
}

pub fn new_txn_serial_with_nonce(
	actor_name: &[u8],
	bytes: Vec<u8>,
	nonce: u64,
) -> Result<TxnSerial> {
	Ok(TxnSerial::new(actor_name.to_vec(), bytes, nonce, u64::MAX))
}

pub async fn query_auth_ops_bytes(auth: AuthKey, gas_limit: u64) -> Result<Vec<u8>> {
	if auth == GOD_MODE_AUTH_KEY {
		error!("If authkey is GOD MODE, use generate_god_mode_ops_bytes instead");
	}
	let auth_bytes = serialize(&auth)?;
	let req = QueryAuthOpsRequest {
		auth_key: auth_bytes,
	};
	let buf = encode_protobuf(req)?;
	let res_bytes = call(
		RegId::Static(codec::NAME).inst(0),
		codec::QueryAuthOpsBytesRequest(buf),
	)
	.await?;
	let (auth_ops, new_expire): (Vec<TokenAuthOp>, u128) = deserialize(res_bytes.0)?;
	send_tx_new_auth_key_expired(&auth, new_expire, gas_limit).await?;
	let auth_ops_bytes = serialize(&auth_ops)?;
	Ok(auth_ops_bytes)
}

pub async fn send_tx_new_auth_key_expired(
	auth: &AuthKey,
	new_expire: u128,
	gas_limit: u64,
) -> Result<()> {
	info!(
		"send_tx_new_auth_key_expired => {:?} | {:?}",
		auth, new_expire
	);
	if new_expire < 1 {
		info!("No need to send extend authkey txn.");
		return Ok(());
	}
	let txn = TappstoreTxn::ExtendAuthkey {
		auth_key: *auth,
		new_expire,
	};
	send_transaction_locally(
		&TxnSerial::new(
			tea_system_actors::tappstore::NAME.to_vec(),
			serialize(&txn)?,
			random_u64().await?,
			gas_limit,
		),
		vec![],
		true,
		|tsid| {
			Box::pin(async move {
				info!("send_tx_new_auth_key_expired result: {:?}", tsid);
				Ok(())
			})
		},
	)
	.await
}

pub async fn read_bonding_balance(
	account: Account,
	ctx: Vec<u8>,
	conflict_mode: ReadConflictMode,
) -> Result<(Balance, Vec<u8>)> {
	let res_buf = call(
		RegId::Static(codec::NAME).inst(0),
		codec::ReadTokenBalanceRequest(encode_protobuf(ReadTokenBalanceRequest {
			account: serialize(&account)?,
			ctx,
			conflict_mode: serialize(&conflict_mode)?,
		})?),
	)
	.await?;
	let res = ReadTokenBalanceResponse::decode(res_buf.0.as_slice())?;
	Ok((deserialize(&res.amount)?, res.ctx))
}

pub async fn read_tea_balance(
	ctx: Vec<u8>,
	account: Account,
	conflict_mode: ReadConflictMode,
) -> Result<(Balance, Vec<u8>)> {
	let res_buf = call(
		RegId::Static(codec::NAME).inst(0),
		codec::ReadTeaBalanceRequest(encode_protobuf(ReadTeaBalanceRequest {
			ctx,
			acct: serialize(&account)?,
			conflict_mode: serialize(&conflict_mode)?,
		})?),
	)
	.await?;
	let res = ReadTeaBalanceResponse::decode(res_buf.0.as_slice())?;
	Ok((deserialize(&res.balance_bytes)?, res.ctx))
}

pub async fn query_tea_balance(token_id: TokenId, account: Account) -> Result<Balance> {
	let res_buf = call(
		RegId::Static(codec::NAME).inst(0),
		codec::QueryTeaBalanceRequest(encode_protobuf(QueryTeaBalanceRequest {
			acct: serialize(&account)?,
			token_id: serialize(&token_id)?,
		})?),
	)
	.await?;
	let res = QueryTeaBalanceResponse::decode(res_buf.0.as_slice())?;
	deserialize(res.balance_bytes).err_into()
}

pub async fn query_tea_deposit_balance(token_id: TokenId, account: Account) -> Result<Balance> {
	let res_buf = call(
		RegId::Static(codec::NAME).inst(0),
		codec::QueryTeaDepositBalanceRequest(encode_protobuf(QueryTeaBalanceRequest {
			acct: serialize(&account)?,
			token_id: serialize(&token_id)?,
		})?),
	)
	.await?;
	let res = QueryTeaBalanceResponse::decode(res_buf.0.as_slice())?;
	let balance = deserialize(res.balance_bytes)?;
	Ok(balance)
}

pub async fn query_allowance(token_id: &TokenId, address: &Account) -> Result<Balance> {
	let req = QueryAllowanceRequest {
		token_id: serialize(token_id)?,
		address: serialize(address)?,
	};
	let buf = encode_protobuf(req)?;
	let res_buf = call(
		RegId::Static(codec::NAME).inst(0),
		codec::QueryAllowanceRequest(buf),
	)
	.await?;
	let res = QueryAllowanceResponse::decode(res_buf.0.as_slice())?;
	let allowance: Balance = deserialize(res.allowance)?;
	Ok(allowance)
}

/// this is the in-transaction version of get_bonding_total_supply. it is
/// used inside a transaction with ctx live.
/// beause there might be some uncommitted add and sub, the get function wont
/// have those changes included. but the read function will
pub async fn read_bonding_total_supply(ctx: Vec<u8>) -> Result<(Balance, Vec<u8>)> {
	let res_buf = call(
		RegId::Static(codec::NAME).inst(0),
		codec::ReadBondingTotalSupplyRequest(encode_protobuf(ReadBondingTotalSupplyRequest {
			ctx,
		})?),
	)
	.await?;
	let res = ReadBondingTotalSupplyResponse::decode(res_buf.0.as_slice())?;
	Ok((deserialize(&res.total_supply)?, res.ctx))
}

pub async fn burn_bonding_token(
	account: Account,
	amount: Balance,
	ctx: Vec<u8>,
) -> Result<Vec<u8>> {
	let res_buf = call(
		RegId::Static(codec::NAME).inst(0),
		codec::BondingBurnRequest(encode_protobuf(BondingBurnRequest {
			ctx,
			account: serialize(&account)?,
			amount: serialize(&amount)?,
		})?),
	)
	.await?;
	let res = BondingBurnResponse::decode(res_buf.0.as_slice())?;
	Ok(res.ctx)
}

pub async fn transfer_bonding_token(
	from_acct: Account,
	to_acct: Account,
	amount: Balance,
	ctx: Vec<u8>,
) -> Result<Vec<u8>> {
	let res_buf = call(
		RegId::Static(codec::NAME).inst(0),
		codec::BondingTransferRequest(encode_protobuf(BondingTransferRequest {
			ctx,
			from_acct: serialize(&from_acct)?,
			to_acct: serialize(&to_acct)?,
			amount: serialize(&amount)?,
		})?),
	)
	.await?;
	let res = BondingTransferResponse::decode(res_buf.0.as_slice())?;
	Ok(res.ctx)
}

pub async fn get_reserved_token_balance(token_id: TokenId) -> Result<Balance> {
	let res_buf = call(
		RegId::Static(codec::NAME).inst(0),
		codec::GetTokenReservedBalanceRequest(encode_protobuf(GetTokenReservedBalanceRequest {
			token_id: serialize(&token_id)?,
		})?),
	)
	.await?;
	let res = GetTokenReservedBalanceResponse::decode(res_buf.0.as_slice())?;
	deserialize(res.amount).err_into()
}

/// This get function can only be called from a pure query, not inside a txn
/// If in side a txn, we will need to use read_bonding_total_supply with a
/// token_ctx as parameter
pub async fn get_bonding_total_supply(token_id: TokenId) -> Result<Balance> {
	let res_buf = call(
		RegId::Static(codec::NAME).inst(0),
		codec::GetBondingTotalSupplyRequest(encode_protobuf(GetTokenTotalSupplyRequest {
			token_id: serialize(&token_id)?,
		})?),
	)
	.await?;
	let res = GetTokenTotalSupplyResponse::decode(res_buf.0.as_slice())?;
	deserialize(res.amount).err_into()
}

pub async fn in_app_purchase(
	from_account: &Account,
	amount: &Balance,
	tappstore_ctx: Vec<u8>,
	payee_ctx: Vec<u8>,
) -> Result<(Vec<u8>, Vec<u8>)> {
	warn!("need some kind of auth, make sure bad actor cannot call this function from unauth use case");

	let req = InAppPurchaseRequest {
		address: serialize(from_account)?,
		amount: serialize(amount)?,
		tappstore_ctx,
		payee_ctx,
	};
	let buf = encode_protobuf(req)?;
	let res_buf = call(
		RegId::Static(codec::NAME).inst(0),
		codec::InAppPurchaseRequest(buf),
	)
	.await?;
	let res = InAppPurchaseResponse::decode(res_buf.0.as_slice())?;
	Ok((res.tappstore_ctx, res.payee_ctx))
}

pub async fn set_allowance(token_id: &TokenId, address: &Account, amount: &Balance) -> Result<()> {
	let req = SetAllowanceRequest {
		token_id: serialize(token_id)?,
		address: serialize(address)?,
		amount: serialize(amount)?,
	};
	let buf = encode_protobuf(req)?;
	let res_buf = call(
		RegId::Static(codec::NAME).inst(0),
		codec::SetAllowanceRequest(buf),
	)
	.await?;
	SetAllowanceResponse::decode(res_buf.0.as_slice())?;
	Ok(())
}

/// check if an account has enough requried_amt balance. return true if yes. else false
pub async fn verify_enough_account_balance(
	acct: &Account,
	ctx: Vec<u8>,
	required_amt: &Balance,
) -> Result<bool> {
	let (balance, _) = read_tea_balance(ctx, *acct, ReadConflictMode::BothConflict).await?;

	if balance < *required_amt {
		Ok(false)
	// return Err("not_enough_balance_postmessage".into());
	} else {
		Ok(true)
	}
}

pub async fn pay_miner_gas(
	miner_token_id: &TokenId, // miner cml entity id
	from_account: &Account,
	amount: &Balance,
	tappstore_ctx: Vec<u8>,
	payee_ctx: Vec<u8>,
) -> Result<(Vec<u8>, Vec<u8>)> {
	warn!("need some kind of auth, make sure bad actor cannot call this function from unauth use case");

	let req = PayMinerGasRequest {
		token_id: serialize(miner_token_id)?,
		address: serialize(from_account)?,
		amount: serialize(amount)?,
		tappstore_ctx,
		payee_ctx,
	};
	let buf = encode_protobuf(req)?;
	let res_buf = call(
		RegId::Static(codec::NAME).inst(0),
		codec::PayMinerGasRequest(buf),
	)
	.await?;
	let res = PayMinerGasResponse::decode(res_buf.0.as_slice())?;
	Ok((res.tappstore_ctx, res.payee_ctx))
}

/// return balance in bytes and tsid in bytes
pub async fn query_token_balance(token_id: TokenId, account: Account) -> Result<Balance> {
	let res_buf = call(
		RegId::Static(codec::NAME).inst(0),
		codec::QueryTokenBalanceRequest(encode_protobuf(QueryTokenBalanceRequest {
			token_id: serialize(&token_id)?,
			acct: serialize(&account)?,
		})?),
	)
	.await?;
	let res = QueryTokenBalanceResponse::decode(res_buf.0.as_slice())?;
	deserialize(res.balance_bytes).err_into()
}
