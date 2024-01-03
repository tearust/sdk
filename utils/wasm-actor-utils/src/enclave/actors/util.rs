use crate::enclave::{
	actors::enclave::generate_random,
	error::{Error, Layer1Errors, Result},
};
use primitive_types::H160;
use tea_sdk::IntoGlobal;

use super::crypto::sha256;

pub const FIXED_TEA_ID_LEN: usize = 32;

/// Transform non-length compatible u8 to tea_id ([u8; 32])
pub fn to_fixed_len_tea_id(buf: &[u8]) -> Result<[u8; FIXED_TEA_ID_LEN]> {
	if buf.len() < FIXED_TEA_ID_LEN {
		return Err(Layer1Errors::BufferLengthMismatch(FIXED_TEA_ID_LEN, buf.len()).into());
	}

	let mut result = [0u8; FIXED_TEA_ID_LEN];
	result.copy_from_slice(&buf[0..FIXED_TEA_ID_LEN]);
	Ok(result)
}

/// Transform u8 buffer to u128.
/// This is usually done for Ts in the system.
pub fn u128_from_le_buffer(data: &[u8]) -> Result<u128> {
	const U128_LENGTH: usize = 16;

	if data.len() < U128_LENGTH {
		return Err(Layer1Errors::U128LengthMismatch(U128_LENGTH).into());
	}

	let mut u128_buf = [0u8; U128_LENGTH];
	u128_buf.copy_from_slice(&data[0..U128_LENGTH]);
	Ok(u128::from_le_bytes(u128_buf))
}

/// Transform H160 address to string.
/// In the system, a user's wallet address as well as token id are both H160.
pub fn h160_to_string(data: H160) -> Result<String> {
	let fmt = format!("{data:?}");
	Ok(fmt)
}

/// Transform string to H160
pub fn str_to_h160(data: &str) -> Result<H160> {
	Ok(data
		.parse()
		.map_err(|e: fixed_hash::rustc_hex::FromHexError| Error::ParseAddress(e.to_string()))?)
}

/// Format and transform a base string to a H160 address.
/// In the TEA system, it will be an email string.
pub async fn email_to_h160(email: &str) -> Result<H160> {
	let hash = sha256(email.to_lowercase().as_bytes().to_vec()).await?;

	let bb: &[u8; 20] = &hash[0..20].try_into().into_g::<Error>()?;
	let acct = H160::from(bb);

	Ok(acct)
}

/// Generate a random length u8 string with the system.
pub async fn randome_number_by_len(len: u8) -> Result<String> {
	let len = u32::from(len);
	let mut rs: Vec<String> = Vec::new();
	let seed = generate_random(len).await?;
	for n in seed {
		let s = n % 10;
		rs.push(s.to_string());
	}
	Ok(rs.join(""))
}

/// Select random count members from an array.
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
