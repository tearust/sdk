use crate::enclave::{
	actors::enclave::generate_random,
	error::{Layer1Errors, Result},
};
use primitive_types::H160;

use super::crypto::sha256;

pub const FIXED_TEA_ID_LEN: usize = 32;

pub fn to_fixed_len_tea_id(buf: &[u8]) -> Result<[u8; FIXED_TEA_ID_LEN]> {
	if buf.len() < FIXED_TEA_ID_LEN {
		return Err(Layer1Errors::BufferLengthMismatch(FIXED_TEA_ID_LEN, buf.len()).into());
	}

	let mut result = [0u8; FIXED_TEA_ID_LEN];
	result.copy_from_slice(&buf[0..FIXED_TEA_ID_LEN]);
	Ok(result)
}

pub fn u128_from_le_buffer(data: &[u8]) -> Result<u128> {
	const U128_LENGTH: usize = 16;

	if data.len() < U128_LENGTH {
		return Err(Layer1Errors::U128LengthMismatch(U128_LENGTH).into());
	}

	let mut u128_buf = [0u8; U128_LENGTH];
	u128_buf.copy_from_slice(&data[0..U128_LENGTH]);
	Ok(u128::from_le_bytes(u128_buf))
}

pub fn h160_to_string(data: H160) -> Result<String> {
	let fmt = format!("{data:?}");
	Ok(fmt)
}

pub fn str_to_h160(data: &str) -> Result<H160> {
	Ok(data.parse()?)
}

pub async fn email_to_h160(email: &str) -> Result<H160> {
	let hash = sha256(email.to_lowercase().as_bytes().to_vec()).await?;

	let bb: &[u8; 20] = &hash[0..20].try_into()?;
	let acct = H160::from(bb);

	Ok(acct)
}

pub async fn random_select<T>(mut select_array: Vec<T>, select_count: usize) -> Result<Vec<T>> {
	if select_array.len() < select_count {
		return Ok(select_array);
	}

	let seeds = generate_seeds(select_count).await?;
	Ok(take_members_from_random_seed(&mut select_array, &seeds))
}

async fn generate_seeds(count: usize) -> Result<Vec<u32>> {
	const SEED_SIZE: usize = 4;
	let mut result = vec![];
	let mut seed_buf = [0u8; SEED_SIZE];
	for _ in 0..count {
		let rand_buf = generate_random(SEED_SIZE as u32).await?;
		seed_buf.copy_from_slice(&rand_buf[0..SEED_SIZE]);
		result.push(u32::from_le_bytes(seed_buf));
	}

	Ok(result)
}

fn take_members_from_random_seed<T>(select_array: &mut Vec<T>, seeds: &[u32]) -> Vec<T> {
	let mut result = Vec::new();
	seeds.iter().for_each(|seed| {
		let index = *seed as usize % select_array.len();
		result.push(select_array.remove(index));
	});
	result
}
