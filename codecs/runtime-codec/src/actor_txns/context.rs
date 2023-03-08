use self::concurrent::ConcurrentBalances;
use crate::actor_txns::auth::AllowedOp;
use crate::actor_txns::error::{Result, TxnError};
use crate::actor_txns::{auth::TokenAuthOp, tsid::Tsid};
use crate::tapp::{Account, AuthKey, Balance, TokenId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use tea_sdk::deserialize;

pub mod concurrent;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TappStorageType {
	AesKey,
	Profile,
	AuthKey(AuthKey),
	SessionKey(TokenId, Account),
	TappStoreKey,
	FailedPayments,
}

/// ReadConflictMode is used to determine if other context have a Credit or Debit operation, will that cause
/// this READ operation invalid.
/// For example, if I read an account balance, and if it is greater than a certain amount then running my logic.
/// In this case, if there is another concurrent credit operation, my logic is still valid. Because
/// credit will just increase the account balance. my original logic "greater than" is still valid.
/// But if there was a Debit operation, my logic might be "invalid" because the debit reduce the balance, that my
/// "greater than" logic might be fail. In this case, the read is conflict. It may cause a rerun when merge
/// Those four enum values means in what situation, my read will be valid / invalid
/// These values are important when running a context merge or rebase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadConflictMode {
	CreditOk,
	DebitOk,
	BothOk,
	BothConflict,
}

pub trait CheckConflict {
	fn check_conflict(&self, other: &Self) -> Result<()>;
}

pub trait Merge {
	fn merge(&mut self, other: &Self) -> Result<()>;
}

pub trait AssetContext {
	fn add_token_read(&mut self, acct: Account, conf_mode: ReadConflictMode);

	fn add_token_add(&mut self, acc: Account, amount: Balance);

	fn add_token_subtract(&mut self, acc: Account, amount: Balance);

	fn accumulate_balance(&self, amount: Balance, acct: Account) -> Result<Balance>;

	fn accumulate_total_supply(&self, total_supply: Balance) -> Result<Balance>;

	fn get_token_adds(&self) -> &HashMap<Account, Vec<Balance>>;

	fn get_token_subtracts(&self) -> &HashMap<Account, Vec<Balance>>;

	fn add_hidden_add(&mut self, amount: Balance) -> Result<Balance>;

	fn add_hidden_subtract(&mut self, amount: Balance) -> Result<Balance>;

	fn get_hidden_add(&self) -> Balance;

	fn get_hidden_subtract(&self) -> Balance;
}

pub trait Context<C>: CheckConflict
where
	C: AssetContext,
{
	/// What tokenid is this context related. Every context can only associate
	/// with one token id.
	/// If there is cross token transaction, there must be multiple contexts
	fn get_token_id(&self) -> TokenId;

	/// Get the tsid of the tsid of the txn when open this context
	fn get_tsid(&self) -> Tsid;

	/// Base is the tsid of txn, that this txn is based on. Based on means that when my txn
	/// is executing, the state is at this base tsid. This base tsid is the latest state version
	/// I have.
	/// Of course, due to concurrency, there might be other txn executing in different replica
	/// of threads. That's why there could be conflict that need to merge or rebase
	fn get_base(&self) -> Tsid;

	fn get_storage(&self, storage_type: TappStorageType) -> Result<&Option<Vec<u8>>>;

	fn add_storage_read(&mut self, storage_type: TappStorageType);

	fn set_storage(&mut self, storage_type: TappStorageType, value: Option<Vec<u8>>);

	fn get_storage_sets(&self) -> &HashMap<TappStorageType, VecDeque<Option<Vec<u8>>>>;

	/// Then where is a fork. For example, my context tsid=60 is executed when 50 was the latest
	/// tsid as far as I knew at that moment. but some other replica is also executing
	/// another txn has the same base (50). Other txn happens earlier than mine with tsid = 55.
	/// In this case, when other replica and mine sync the state, I will need to rebase mine since
	/// my tsid 60 is later than others tsid50. Mine should rebase to the base of 55 instead of
	/// original 50
	/// This operation is called rebase
	/// In most cases, rebase can be a simple merge as long as there is no conflict
	/// But if there is, in most cases, a rerun is not avoidable.
	/// When doing the rerun, my txn context will be based on the latest tsid I am aware of,
	/// that is tsid 55. So my new base would be 55.
	/// if there is no need to rerun in case of conflict free, a simple CRDT merge
	/// can be done to rebase.
	fn rebase(&mut self, other: &Self) -> Result<()>;

	// /// Check authorized operation matches or not
	fn check_auth(&self, ask_acct: Account, ask_ops: Vec<AllowedOp>) -> Result<()>;

	fn bonding_context(&self) -> &C;

	fn bonding_context_mut(&mut self) -> &mut C;

	fn tea_context(&self) -> &C;

	fn tea_context_mut(&mut self) -> &mut C;

	fn deposit_context(&self) -> &C;

	fn deposit_context_mut(&mut self) -> &mut C;

	fn allowance_context(&self) -> &C;

	fn allowance_context_mut(&mut self) -> &mut C;
}

/// TokenContext implement the context trait
/// it store all changes a txn made before actually commit to state
/// When there is a read inside a txn logic, it will check its context
/// first, in case there is a prior change made by prior logic
/// when write, all write in context without changing the actual
/// state, until final commit
/// if commit fail, nothing will change in the state.
/// if commit succeed, everything will be write into the state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenContext {
	/// Token Id, this context has and has only one tokenid related
	/// If a transaction need to read/write multiple token (FT or NFT)
	/// it needs to start multiple contexts. Every context associate
	/// to a FT/NFT token
	pub tid: TokenId,
	/// the tsid of the txn the context is associated. Every
	/// context is associated with a specific txn
	pub tsid: Tsid,

	/// every context is based on an existing state or a previous context,
	/// this base tsid is the existing state.tsid or the previous context.Tsid
	/// all context changes are relatively to this base.
	/// this context can only be committed when the state.tsid is my base, or
	/// I can merge with a previous context into a bigger context using its
	/// tsid as my base. this is called merge.  
	/// if this is not the case, a rebase is needed before commit
	base: Tsid,

	/// a hashset of a group of reads on tapp storage
	reads_storage: HashSet<TappStorageType>,

	/// Storage related operations, since we treat the storage as a atomic item, and we cannot
	/// get a "increment value" for the value with buffer, so here add a set of exclusive operations:
	/// add, update, and remove
	storage_changes: HashMap<TappStorageType, VecDeque<Option<Vec<u8>>>>,

	tea: ConcurrentBalances,
	deposit: ConcurrentBalances,
	bonding: ConcurrentBalances,
	allowance: ConcurrentBalances,
	/// allowed ops. Note: Because the stored procedure actor may
	/// change the context. We do NOT trust the context when the
	/// state accept the context to commit. we HAVE TO verify again
	/// the auth ops from the auth key at the time of commit.
	/// So it is possible that the context cannot get commited at
	/// the end due to auth fail. That usually caused by the
	/// stored procedure developer ignore the checking of auth.
	/// we cannot enforce the stored procedure developer to respect
	/// the auth checking rule, but we can verify again at the commit
	/// time. so it may be refused to commit to state at the end.
	auth_ops: Vec<TokenAuthOp>,
}

impl CheckConflict for TokenContext {
	fn check_conflict(&self, other: &Self) -> Result<()> {
		self.check_base_conflict(other)?;
		self.check_storage_conflict(other)?;
		self.tea.check_conflict(&other.tea)?;
		self.deposit.check_conflict(&other.deposit)?;
		self.bonding.check_conflict(&other.bonding)?;
		// self.bonding_reserved
		// 	.check_conflict(&other.bonding_reserved)?;
		Ok(())
	}
}

impl Context<ConcurrentBalances> for TokenContext {
	/// What token (FT/NFT) is this context associate?
	fn get_token_id(&self) -> TokenId {
		self.tid
	}

	fn get_tsid(&self) -> Tsid {
		self.tsid
	}
	fn get_base(&self) -> Tsid {
		self.base
	}

	fn get_storage(&self, storage_type: TappStorageType) -> Result<&Option<Vec<u8>>> {
		if let Some(v) = self.storage_changes.get(&storage_type) {
			if let Some(element) = v.back() {
				return Ok(element);
			}
		}

		Err(TxnError::StorageIsEmpty(storage_type).into())
	}
	fn add_storage_read(&mut self, storage_type: TappStorageType) {
		self.reads_storage.insert(storage_type);
	}

	fn set_storage(&mut self, storage_type: TappStorageType, value: Option<Vec<u8>>) {
		if let Some(changes) = self.storage_changes.get_mut(&storage_type) {
			changes.push_back(value);
		} else {
			self.storage_changes
				.insert(storage_type, VecDeque::from(vec![value]));
		}
	}
	fn get_storage_sets(&self) -> &HashMap<TappStorageType, VecDeque<Option<Vec<u8>>>> {
		&self.storage_changes
	}

	/// Rebase is a more commonly used conflict resolver than merge.
	/// Rebase will not swallow previous context, instead, it change the base of
	/// my self. So that instead of a fork to two sibling, I will change
	/// my base to you, so that the sequence would be: you based on parent,
	/// I based on you. so there will be no conflict.
	/// You can commit first. After your commit, the state machien tsid will be yours.
	/// then I commit. Because my base is already rebased yours, there is no
	/// conflict when I commit. The state machine is happy accepting both of us.
	fn rebase(&mut self, other: &Self) -> Result<()> {
		self.check_conflict(other)?;

		if self.tid != other.tid {
			// rebase free when contexts from different token id
			return Ok(());
		}

		// right now , there is no conflict, I can simple rebase to other tsid
		// if any of the conditions above not met, we cannot run into this step
		// and a rerun could be the only solution instead of rebase
		self.base = other.get_tsid();
		Ok(())
	}

	fn check_auth(&self, _acct: Account, _ask_ops: Vec<AllowedOp>) -> Result<()> {
		// todo disable check auth for now
		Ok(())
		// // let token_auth_ops = self.auth_ops;
		// // checking debit auth ops
		// debug!(
		// 	"context check auth ask_ops: {:?}, user_login_auth_ops: {:?}",
		// 	&ask_ops, &self.auth_ops
		// );
		// debug!("tid:{:?} - {:?} ", &self.tid, tappstore_id()?);
		// debug!("acct:{:?}", acct);
		// for ask_op in ask_ops {
		// 	let mut auth_ok = false;
		// 	for token_auth_op in &self.auth_ops {
		// 		// if token_auth_op.acct == acct  // this will failed when sell_token.
		// 		if self.tid == tappstore_id()? || token_auth_op.token_id == tappstore_id()? {
		// 			debug!("now check token_auth_op.check_auth");
		// 			// token_auth_op.check_auth(acct, self.tid, ask_op)?;
		// 			token_auth_op.check_auth(acct, tappstore_id()?, ask_op)?;
		// 			auth_ok = true;
		// 		}
		// 	} //for token auth ops
		// 	if auth_ok == false {
		// 		error!("auth check failed");
		// 		return Err(ContextError::AuthCheckFailed(self.tid, acct, ask_op));
		// 	}
		// } // for ask ops
		// Ok(())
	}

	fn bonding_context(&self) -> &ConcurrentBalances {
		&self.bonding
	}

	fn bonding_context_mut(&mut self) -> &mut ConcurrentBalances {
		&mut self.bonding
	}

	fn tea_context(&self) -> &ConcurrentBalances {
		&self.tea
	}

	fn tea_context_mut(&mut self) -> &mut ConcurrentBalances {
		&mut self.tea
	}

	fn deposit_context(&self) -> &ConcurrentBalances {
		&self.deposit
	}

	fn deposit_context_mut(&mut self) -> &mut ConcurrentBalances {
		&mut self.deposit
	}

	fn allowance_context(&self) -> &ConcurrentBalances {
		&self.allowance
	}

	fn allowance_context_mut(&mut self) -> &mut ConcurrentBalances {
		&mut self.allowance
	}
}

impl TokenContext {
	pub fn new(tsid: Tsid, base: Tsid, tid: TokenId, auth_ops_bytes: &[u8]) -> Result<Self> {
		let auth_ops: Vec<TokenAuthOp> = if auth_ops_bytes.is_empty() {
			//TODO: We should remove all zero length auth_ops_bytes cases
			Vec::new()
		} else {
			deserialize(auth_ops_bytes)?
		};

		Ok(TokenContext {
			tsid,
			tid,
			base,
			auth_ops,
			..Default::default()
		})
	}

	pub fn new_slim(tsid: Tsid, base: Tsid, tid: TokenId) -> Self {
		TokenContext {
			tsid,
			tid,
			base,
			..Default::default()
		}
	}

	fn check_base_conflict(&self, other: &Self) -> Result<()> {
		if self.tsid == other.get_tsid() {
			return Err(TxnError::ShouldNotCheckSameTsid.into());
		}
		// you do not care about the later txn because you will
		// always be ahead of it.
		// but it will be care about you, because it
		// will need to rebase or merge you.
		if self.tsid < other.get_tsid() {
			return Err(TxnError::ShouldNotCheckConflictWithLaterTsid.into());
		}
		// if the you and other are based on two different
		// tsid, there is no way to determine conflicts
		// because you and other are in two forked branches
		// both of you have to trace back to the common parent
		// where the fork starts, then merge or rebase from that point first
		if other.get_base() != self.base {
			return Err(TxnError::BaseNotMatchError.into());
		}
		Ok(())
	}

	fn check_storage_conflict(&self, other: &Self) -> Result<()> {
		let mut touched_storage = self.reads_storage.clone();
		for (t, _) in self.storage_changes.iter() {
			touched_storage.insert(*t);
		}

		for t in touched_storage {
			if other.reads_storage.contains(&t) || other.storage_changes.contains_key(&t) {
				return Err(TxnError::StorageHasBeTouched(t).into());
			}
		}

		Ok(())
	}

	pub fn get_current_tsid(&self) -> Tsid {
		self.get_tsid()
	}

	pub fn log_from_bytes(bytes: &[u8]) -> Result<String> {
		let ctx: Self = deserialize(bytes)?;
		Ok(format!("{:?}", &ctx))
	}
	pub fn log_allowance_from_bytes(bytes: &[u8]) -> Result<String> {
		let ctx: Self = deserialize(bytes)?;
		Ok(format!(
			"token_id: {:#?}, allowance: {:?}",
			&ctx.get_token_id(),
			&ctx.allowance_context()
		))
	}
	pub fn log_tea_from_bytes(bytes: &[u8]) -> Result<String> {
		let ctx: Self = deserialize(bytes)?;
		Ok(format!(
			"token_id: {:#?}, tea: {:?}",
			&ctx.get_token_id(),
			&ctx.tea_context()
		))
	}
	pub fn log_deposit_from_bytes(bytes: &[u8]) -> Result<String> {
		let ctx: Self = deserialize(bytes)?;
		Ok(format!(
			"token_id: {:#?}, deposit: {:?}",
			&ctx.get_token_id(),
			&ctx.deposit_context()
		))
	}
	pub fn log_bonding_from_bytes(bytes: &[u8]) -> Result<String> {
		let ctx: Self = deserialize(bytes)?;
		Ok(format!(
			"token_id: {:#?}, bonding: {:?}",
			&ctx.get_token_id(),
			&ctx.bonding_context()
		))
	}
}
