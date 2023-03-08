pub mod adapter;
pub mod console;
pub mod crypto;
pub mod env;
pub mod http;
pub mod ipfs;
pub mod ipfs_out;
pub mod kvp;
pub mod layer1;
pub mod libp2p;
pub mod nitro;
pub mod persist;
pub mod replica;
pub mod state_receiver;
pub mod token_state;

pub const GENERAL_REQUEST_PREFIX: &str = "request";
pub const GENERAL_POST_PREFIX: &str = "post";
pub const GENERAL_REPLY_PREFIX: &str = "reply";
pub const GENERAL_INTERFACE_PREFIX: &str = "actor";

/// The general reply subject
pub fn general_callback_reply(actor: &str, seq_number: u64) -> String {
	format!("{GENERAL_REPLY_PREFIX}.{actor}.{seq_number}")
}

pub fn general_request_subject(actor: &str) -> String {
	format!("{GENERAL_REQUEST_PREFIX}.{actor}")
}

pub fn general_post_subject(actor: &str) -> String {
	format!("{GENERAL_POST_PREFIX}.{actor}")
}

pub fn general_actor_interface(actor: &str, action: &str) -> String {
	format!("{GENERAL_INTERFACE_PREFIX}.{actor}.{action}")
}

pub fn parse_seq_number_from_reply(reply_to: &str) -> Option<u64> {
	let num_str = reply_to.split('.').nth(2);
	match num_str {
		Some(s) => match s.parse() {
			Ok(num) => Some(num),
			Err(_) => None,
		},
		None => None,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_seq_number_works() {
		let seq_number = 333;
		assert_eq!(
			parse_seq_number_from_reply(&general_callback_reply("actor", seq_number)),
			Some(seq_number)
		);

		assert_eq!(parse_seq_number_from_reply("simple_text"), None);
		assert_eq!(
			parse_seq_number_from_reply(&format!(
				"{}.{}.{}",
				GENERAL_REPLY_PREFIX, "actor", "text"
			)),
			None
		);
	}
}
