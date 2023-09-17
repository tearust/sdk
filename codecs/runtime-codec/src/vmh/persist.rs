use crate::tapp::{statement::TypedStatement, Account, Ts};
use crate::tapp::{TimestampShort, TxnHashFileNumber};

use crate::vmh::error::{PersistCheck, Result};

pub const FIXED_PREFIX_LENGTH: usize = 8;
pub const PADDING_BYTE: u8 = b'0';
pub const CHECKPOINT_PREFIX: &str = "check_pt";

pub fn persist_prefix(prefix: &str) -> Result<Vec<u8>> {
	let mut rtn = prefix.as_bytes().to_vec();
	if rtn.len() > FIXED_PREFIX_LENGTH {
		return Err(PersistCheck::PrefixTooLong(FIXED_PREFIX_LENGTH, rtn.len()).into());
	}

	rtn.resize(FIXED_PREFIX_LENGTH, PADDING_BYTE);
	Ok(rtn)
}

pub fn key_without_prefix(key: &[u8]) -> Result<Vec<u8>> {
	if key.len() <= FIXED_PREFIX_LENGTH {
		return Err(PersistCheck::KeyTooShort(key.to_vec()).into());
	}

	Ok(key[FIXED_PREFIX_LENGTH..key.len()].to_vec())
}

pub fn check_prefix_length(prefix: &[u8]) -> Result<()> {
	if prefix.len() != FIXED_PREFIX_LENGTH {
		return Err(PersistCheck::PrefixLengthMismatch(FIXED_PREFIX_LENGTH, prefix.len()).into());
	}
	Ok(())
}

pub fn combined_key(key: Vec<u8>, mut prefix: Vec<u8>) -> Vec<u8> {
	prefix.extend(key.iter());
	prefix
}

pub trait Persist {
	fn set(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;

	fn set_with_prefix(&mut self, key: Vec<u8>, prefix: Vec<u8>, value: Vec<u8>) -> Result<()> {
		check_prefix_length(&prefix)?;
		self._set_with_prefix(key, prefix, value)
	}

	fn _set_with_prefix(&mut self, key: Vec<u8>, prefix: Vec<u8>, value: Vec<u8>) -> Result<()>;

	fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;

	fn get_last(&self) -> Result<Option<(Vec<u8>, Vec<u8>)>>;

	/// find with prefix and return matched results
	fn find(
		&self,
		prefix: &[u8],
		options: Option<(u32, u32)>,
		only_key: bool,
	) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
		check_prefix_length(prefix)?;
		self._find(prefix, options, only_key)
	}

	/// find with prefix and return matched results
	fn _find(
		&self,
		prefix: &[u8],
		options: Option<(u32, u32)>,
		only_key: bool,
	) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;

	/// If find the given key then return sequences before/after the key, else return error
	fn find_with_direction(
		&self,
		key: &[u8],
		before: bool,
		options: Option<(u32, u32)>,
		only_key: bool,
	) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;

	fn append_digest(&self, new_item: serde_json::Value) -> Result<()>;

	fn append_statements(&self, statements: Vec<(Vec<TypedStatement>, Ts, String)>) -> Result<()>;

	fn write_file(&self, file_name: &str, data: &[u8]) -> Result<()>;

	fn write_txn_hashes(
		&mut self,
		timestamp: TimestampShort,
		genesis: bool,
		data: &[u8],
		only_touch: bool,
	) -> Result<()>;

	fn find_txn_hashes(&self, timestamp: TimestampShort) -> Result<Vec<u8>>;

	fn find_miss_txn_hash_files(
		&self,
		start_time: TimestampShort,
		end_time: TimestampShort,
	) -> Result<Vec<(TxnHashFileNumber, TxnHashFileNumber)>>;

	fn read_txn_hash_file(&self, num: TxnHashFileNumber) -> Result<Vec<u8>>;

	fn write_txn_hash_file(&mut self, num: TxnHashFileNumber, data: &[u8]) -> Result<()>;

	fn get_pre_file_cid(
		&self,
		timestamp: TimestampShort,
		last_persist_ts: TimestampShort,
		version: u16,
	) -> Result<Option<String>>;

	fn exist_txn_genesis_file(&self) -> Result<bool>;

	fn read_txn_genesis_file(&self) -> Result<Vec<u8>>;

	fn write_txn_genesis_file(&mut self, data: &[u8]) -> Result<()>;

	fn can_txn_hashes_sync(&self) -> Result<bool>;

	fn get_statements(
		&self,
		account_filter: Option<Account>,
		date: Option<chrono::NaiveDate>,
		max_size: u64,
		read_to_end: &mut bool,
	) -> Result<Vec<(TypedStatement, String, String)>>;
}

#[cfg(test)]
mod tests {
	use crate::vmh::error::Result;
	use crate::vmh::persist::{key_without_prefix, persist_prefix};

	#[test]
	fn persist_prefix_works() -> Result<()> {
		assert_eq!(persist_prefix("12345678")?, "12345678".as_bytes().to_vec());
		assert_eq!(persist_prefix("12345")?, "12345000".as_bytes().to_vec());
		assert_eq!(persist_prefix("fffff")?, "fffff000".as_bytes().to_vec());
		persist_prefix("123456789").unwrap_err();

		Ok(())
	}

	#[test]
	fn key_without_prefix_works() -> Result<()> {
		assert_eq!(
			key_without_prefix("123456789".as_bytes())?,
			"9".as_bytes().to_vec()
		);
		key_without_prefix("12345678".as_bytes()).unwrap_err();
		key_without_prefix("1234567".as_bytes()).unwrap_err();

		Ok(())
	}
}
