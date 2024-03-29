use super::{AssetContext, CheckConflict, IsBalanceRelated, Merge, ReadConflictMode};
use crate::actor_txns::error::{ContextError, Error, Result};
use crate::tapp::{Account, Balance};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::{HashMap, HashSet};
use tea_sdk::serialize;

#[doc(hidden)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConcurrentBalances {
	token_add: HashMap<Account, Vec<Balance>>,
	token_subtract: HashMap<Account, Vec<Balance>>,
	read_add: HashSet<Account>,
	read_subtract: HashSet<Account>,
	hidden_add: Balance,
	hidden_subtract: Balance,
}

impl CheckConflict for ConcurrentBalances {
	fn check_conflict(&self, other: &Self) -> crate::actor_txns::error::Result<()> {
		self.check_partial_conflict(other)
	}
}

impl IsBalanceRelated for ConcurrentBalances {
	fn is_balance_related(&self) -> bool {
		!self.token_add.is_empty() || !self.token_subtract.is_empty()
	}
}

impl Merge for ConcurrentBalances {
	fn merge(&mut self, other: &Self) -> Result<()> {
		for (acct, debits) in other.token_subtract.iter() {
			match self.token_subtract.get_mut(acct) {
				Some(self_debit) => self_debit.extend_from_slice(debits),
				None => {
					self.token_subtract.insert(*acct, debits.clone());
				}
			}
		}
		for (acct, credit) in other.token_add.iter() {
			match self.token_add.get_mut(acct) {
				Some(self_credit) => self_credit.extend_from_slice(credit),
				None => {
					self.token_add.insert(*acct, credit.clone());
				}
			}
		}
		Ok(())
	}
}

impl AssetContext for ConcurrentBalances {
	fn add_token_read(&mut self, acct: Account, conf_mode: ReadConflictMode) {
		match conf_mode {
			ReadConflictMode::BothOk => {}
			ReadConflictMode::BothConflict => {
				self.read_add.insert(acct);
				self.read_subtract.insert(acct);
			}
			ReadConflictMode::CreditOk => {
				self.read_subtract.insert(acct);
			}
			ReadConflictMode::DebitOk => {
				self.read_add.insert(acct);
			}
		}
	}

	/// Add an token add operation without checking
	fn add_token_add(&mut self, acc: Account, amount: Balance) {
		append_or_insert_if_newer(acc, amount, &mut self.token_add)
	}

	/// Add an token substract operation without checking
	fn add_token_subtract(&mut self, acc: Account, amount: Balance) {
		append_or_insert_if_newer(acc, amount, &mut self.token_subtract)
	}

	fn accumulate_balance(&self, amount: Balance, acct: Account) -> Result<Balance> {
		let mut total_debit = Balance::zero();
		let mut total_credit = Balance::zero();
		if let Some(debits) = self.token_subtract.get(&acct) {
			for debit in debits {
				total_debit = total_debit
					.checked_add(*debit)
					.ok_or(ContextError::AddOverflow)?;
			}
		}
		if let Some(credits) = self.token_add.get(&acct) {
			for credit in credits {
				total_credit = total_credit
					.checked_add(*credit)
					.ok_or(ContextError::AddOverflow)?;
			}
		}

		amount
			.checked_add(total_credit)
			.ok_or(ContextError::AddOverflow)?
			.checked_sub(total_debit)
			.ok_or_else(|| ContextError::SubtractionOverflow.into())
	}

	fn accumulate_total_supply(&self, total_supply: Balance) -> Result<Balance> {
		let mut total_debit = Balance::zero();
		let mut total_credit = Balance::zero();
		self.token_subtract.iter().try_for_each(|(_, balances)| {
			total_debit = total_debit
				.checked_add(balances_sum(balances)?)
				.ok_or(ContextError::AddOverflow)?;
			Ok(()) as Result<_>
		})?;
		self.token_add.iter().try_for_each(|(_, balances)| {
			total_credit = total_credit
				.checked_add(balances_sum(balances)?)
				.ok_or(ContextError::AddOverflow)?;
			Ok(()) as Result<_>
		})?;

		total_supply
			.checked_add(total_credit)
			.ok_or(ContextError::AddOverflow)?
			.checked_sub(total_debit)
			.ok_or_else(|| ContextError::SubtractionOverflow.into())
	}

	fn get_token_adds(&self) -> &HashMap<Account, Vec<Balance>> {
		&self.token_add
	}

	fn get_token_subtracts(&self) -> &HashMap<Account, Vec<Balance>> {
		&self.token_subtract
	}

	fn add_hidden_add(&mut self, amount: Balance) -> Result<Balance> {
		self.hidden_add = self
			.hidden_add
			.checked_add(amount)
			.ok_or(ContextError::AddOverflow)?;
		Ok(self.hidden_add)
	}

	fn add_hidden_subtract(&mut self, amount: Balance) -> Result<Balance> {
		self.hidden_subtract = self
			.hidden_subtract
			.checked_add(amount)
			.ok_or(ContextError::AddOverflow)?;
		Ok(self.hidden_subtract)
	}

	fn get_hidden_add(&self) -> Balance {
		self.hidden_add
	}

	fn get_hidden_subtract(&self) -> Balance {
		self.hidden_subtract
	}
}

impl ConcurrentBalances {
	fn check_partial_conflict(&self, other: &ConcurrentBalances) -> Result<()> {
		for read in self.read_add.iter() {
			if other.token_add.contains_key(read) {
				return Err(ContextError::ReadWhileCredit.into());
			}
		}

		for read in self.read_subtract.iter() {
			if other.token_subtract.contains_key(read) {
				return Err(ContextError::ReadWhileDebit.into());
			}
		}
		self.check_double_debit(other)
	}

	fn check_double_debit(&self, other: &ConcurrentBalances) -> Result<()> {
		for debit_acc in self.token_subtract.keys() {
			if other.token_subtract.contains_key(debit_acc) {
				return Err(ContextError::DoubleDebit.into());
			}
		}
		Ok(())
	}

	pub fn hash(&self, hasher: &mut sha2::Sha256) -> Result<()> {
		self.hash_balance_map(&self.token_add, hasher)?;
		self.hash_balance_map(&self.token_subtract, hasher)?;
		self.hash_balance_set(&self.read_add, hasher)?;
		self.hash_balance_set(&self.read_subtract, hasher)?;
		self.hash_balance_hash(&self.hidden_add, hasher)?;
		self.hash_balance_hash(&self.hidden_subtract, hasher)?;
		Ok(())
	}

	fn hash_balance_set(&self, set: &HashSet<Account>, hasher: &mut sha2::Sha256) -> Result<()> {
		let mut token_set = set.iter().collect::<Vec<_>>();
		token_set.sort();
		for acc in token_set {
			hasher.update(acc.as_ref());
		}
		Ok(())
	}

	fn hash_balance_map(
		&self,
		map: &HashMap<Account, Vec<Balance>>,
		hasher: &mut sha2::Sha256,
	) -> Result<()> {
		let mut token_map = map.iter().collect::<Vec<_>>();
		token_map.sort_by(|a, b| a.0.cmp(b.0));
		for (acc, amount_list) in token_map {
			hasher.update(acc.as_ref());
			for amount in amount_list {
				self.hash_balance_hash(amount, hasher)?;
			}
		}
		Ok(())
	}

	fn hash_balance_hash(&self, balance: &Balance, hasher: &mut sha2::Sha256) -> Result<()> {
		hasher.update(serialize(balance).map_err(|e| {
			Error::Unnamed(format!(
				"ConcurrentBalances hash_balance_hash error: {:?}",
				e
			))
		})?);
		Ok(())
	}
}

fn append_or_insert_if_newer(
	acc: Account,
	amount: Balance,
	ops: &mut HashMap<Account, Vec<Balance>>,
) {
	if let Some(v) = ops.get_mut(&acc) {
		v.push(amount);
	} else {
		ops.insert(acc, vec![amount]);
	}
}

/// Check_add for balance.
pub fn balances_sum(balances: &[Balance]) -> Result<Balance> {
	let mut sum = Balance::zero();
	for balance in balances {
		sum = sum.checked_add(*balance).ok_or(ContextError::AddOverflow)?;
	}
	Ok(sum)
}

#[cfg(test)]
mod tests {
	use crate::actor_txns::{
		context::{concurrent::ConcurrentBalances, AssetContext, CheckConflict, ReadConflictMode},
		error::ContextError,
	};
	use crate::tapp::{Account, Balance};

	#[test]
	fn accumulate_balance_works() {
		let acc1 = Account::from([11; 20]);
		let mut ctx1 = ConcurrentBalances::default();

		let amount1 = Balance::from(123456);
		ctx1.add_token_add(acc1, amount1);

		let amount2 = Balance::from(32345);
		assert_eq!(
			ctx1.accumulate_balance(amount2, acc1).unwrap(),
			amount1 + amount2
		);

		let amount3 = Balance::from(43212);
		ctx1.add_token_subtract(acc1, amount3);
		assert_eq!(
			ctx1.accumulate_balance(amount2, acc1).unwrap(),
			amount1 + amount2 - amount3
		);
	}

	#[test]
	fn accumulate_balance_corner_cases() {
		let acc1 = Account::from([11; 20]);

		let mut ctx1 = ConcurrentBalances::default();
		ctx1.add_token_add(acc1, 1.into());
		ctx1.add_token_add(acc1, Balance::MAX);
		assert_eq!(
			ctx1.accumulate_balance(0.into(), acc1),
			Err(ContextError::AddOverflow.into())
		);

		ctx1 = ConcurrentBalances::default();
		ctx1.add_token_subtract(acc1, 1.into());
		ctx1.add_token_subtract(acc1, Balance::MAX);
		assert_eq!(
			ctx1.accumulate_balance(0.into(), acc1),
			Err(ContextError::AddOverflow.into())
		);

		ctx1 = ConcurrentBalances::default();
		ctx1.add_token_add(acc1, 1.into());
		assert_eq!(
			ctx1.accumulate_balance(Balance::MAX, acc1),
			Err(ContextError::AddOverflow.into())
		);

		ctx1 = ConcurrentBalances::default();
		ctx1.add_token_add(acc1, 1.into());
		ctx1.add_token_subtract(acc1, 10.into());
		assert_eq!(
			ctx1.accumulate_balance(2.into(), acc1),
			Err(ContextError::SubtractionOverflow.into())
		);
	}

	#[test]
	fn accumulate_total_supply_works() {
		let acc1 = Account::from([11; 20]);
		let acc2 = Account::from([22; 20]);
		let mut ctx1 = ConcurrentBalances::default();

		let amount1 = Balance::from(123456);
		ctx1.add_token_add(acc1, amount1);

		let amount2 = Balance::from(32345);
		ctx1.add_token_add(acc2, amount2);

		let amount3 = Balance::from(111);
		assert_eq!(
			ctx1.accumulate_total_supply(amount3),
			Ok(amount1 + amount2 + amount3)
		);

		let amount4 = Balance::from(43212);
		ctx1.add_token_subtract(acc1, amount4);

		assert_eq!(
			ctx1.accumulate_total_supply(amount3),
			Ok(amount1 + amount2 + amount3 - amount4)
		);
	}

	#[test]
	fn accumulate_total_supply_corner_cases() {
		let acc1 = Account::from([11; 20]);
		let acc2 = Account::from([22; 20]);

		let mut ctx1 = ConcurrentBalances::default();
		ctx1.add_token_add(acc1, 1.into());
		ctx1.add_token_add(acc2, Balance::MAX);
		assert_eq!(
			ctx1.accumulate_total_supply(0.into()),
			Err(ContextError::AddOverflow.into())
		);

		ctx1 = ConcurrentBalances::default();
		ctx1.add_token_subtract(acc1, 100.into());
		ctx1.add_token_subtract(acc2, Balance::MAX);
		assert_eq!(
			ctx1.accumulate_total_supply(0.into()),
			Err(ContextError::AddOverflow.into())
		);

		ctx1 = ConcurrentBalances::default();
		ctx1.add_token_add(acc1, 100.into());
		assert_eq!(
			ctx1.accumulate_total_supply(Balance::MAX),
			Err(ContextError::AddOverflow.into())
		);

		ctx1 = ConcurrentBalances::default();
		ctx1.add_token_add(acc1, 100.into());
		ctx1.add_token_subtract(acc2, 200.into());
		assert_eq!(
			ctx1.accumulate_total_supply(50.into()),
			Err(ContextError::SubtractionOverflow.into())
		);
	}

	#[test]
	fn check_merge_works() {
		let acc1 = Account::from([11; 20]);
		let acc2 = Account::from([22; 20]);
		let mut ctx1 = ConcurrentBalances::default();
		let mut ctx2 = ConcurrentBalances::default();

		ctx1.check_conflict(&ctx2).unwrap();

		// both ok
		ctx1 = Default::default();
		ctx2 = Default::default();
		ctx1.add_token_read(acc1, ReadConflictMode::BothOk);
		ctx2.add_token_add(acc1, 1.into());
		ctx2.add_token_subtract(acc1, 1.into());
		ctx1.check_conflict(&ctx2).unwrap();

		// credit ok
		ctx1 = Default::default();
		ctx2 = Default::default();
		ctx1.add_token_read(acc1, ReadConflictMode::CreditOk);
		ctx2.add_token_add(acc1, 1.into());
		ctx1.check_conflict(&ctx2).unwrap();
		ctx2.add_token_subtract(acc1, 1.into());
		ctx1.check_conflict(&ctx2).unwrap_err();

		// debit ok
		ctx1 = Default::default();
		ctx2 = Default::default();
		ctx1.add_token_read(acc1, ReadConflictMode::DebitOk);
		ctx2.add_token_subtract(acc1, 1.into());
		ctx1.check_conflict(&ctx2).unwrap();
		ctx2.add_token_add(acc1, 1.into());
		ctx1.check_conflict(&ctx2).unwrap_err();

		// both conflict
		ctx1 = Default::default();
		ctx2 = Default::default();
		ctx1.add_token_read(acc1, ReadConflictMode::BothConflict);
		ctx2.add_token_subtract(acc1, 1.into());
		ctx1.check_conflict(&ctx2).unwrap_err();
		ctx2 = Default::default();
		ctx2.add_token_add(acc1, 1.into());
		ctx1.check_conflict(&ctx2).unwrap_err();

		// no conflict with different accounts

		ctx1 = Default::default();
		ctx2 = Default::default();
		ctx1.add_token_read(acc1, ReadConflictMode::BothConflict);
		ctx2.add_token_add(acc2, 1.into());
		ctx2.add_token_subtract(acc2, 1.into());
		ctx1.check_conflict(&ctx2).unwrap();

		// -- reverse should all pass --

		// both ok
		ctx1 = Default::default();
		ctx2 = Default::default();
		ctx2.add_token_read(acc1, ReadConflictMode::BothOk);
		ctx1.add_token_add(acc1, 1.into());
		ctx1.add_token_subtract(acc1, 1.into());
		ctx1.check_conflict(&ctx2).unwrap();

		// credit ok
		ctx1 = Default::default();
		ctx2 = Default::default();
		ctx2.add_token_read(acc1, ReadConflictMode::CreditOk);
		ctx1.add_token_add(acc1, 1.into());
		ctx1.check_conflict(&ctx2).unwrap();
		ctx1.add_token_subtract(acc1, 1.into());
		ctx1.check_conflict(&ctx2).unwrap();

		// debit ok
		ctx1 = Default::default();
		ctx2 = Default::default();
		ctx2.add_token_read(acc1, ReadConflictMode::DebitOk);
		ctx1.add_token_subtract(acc1, 1.into());
		ctx1.check_conflict(&ctx2).unwrap();
		ctx1.add_token_add(acc1, 1.into());
		ctx1.check_conflict(&ctx2).unwrap();

		// both conflict
		ctx1 = Default::default();
		ctx2 = Default::default();
		ctx2.add_token_read(acc1, ReadConflictMode::BothConflict);
		ctx1.add_token_subtract(acc1, 1.into());
		ctx1.check_conflict(&ctx2).unwrap();
		ctx1 = Default::default();
		ctx1.add_token_add(acc1, 1.into());
		ctx1.check_conflict(&ctx2).unwrap();

		// no conflict with different accounts

		ctx1 = Default::default();
		ctx2 = Default::default();
		ctx2.add_token_read(acc1, ReadConflictMode::BothConflict);
		ctx1.add_token_add(acc2, 1.into());
		ctx1.add_token_subtract(acc2, 1.into());
		ctx1.check_conflict(&ctx2).unwrap();
	}

	#[test]
	fn check_double_debit_ok() {
		let acc1 = Account::from([11; 20]);
		let acc2 = Account::from([22; 20]);
		let mut ctx1 = ConcurrentBalances::default();
		let mut ctx2 = ConcurrentBalances::default();

		ctx1.check_double_debit(&ctx2).unwrap();

		// double debit error
		ctx1.token_subtract.insert(acc1, vec![1.into()]);
		ctx2.token_subtract.insert(acc1, vec![Balance::from(2)]);
		ctx1.check_double_debit(&ctx2).unwrap_err();

		// double debit error even if balances is empty
		ctx1.token_subtract.insert(acc1, vec![]);
		ctx2.token_subtract.insert(acc1, vec![]);
		ctx1.check_double_debit(&ctx2).unwrap_err();

		// not error if with different account
		ctx1 = Default::default();
		ctx2 = Default::default();
		ctx1.token_subtract.insert(acc1, vec![1.into()]);
		ctx2.token_subtract.insert(acc2, vec![Balance::from(2)]);
		ctx1.check_double_debit(&ctx2).unwrap();
	}
}
