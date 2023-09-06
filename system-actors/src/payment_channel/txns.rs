use super::error::{NotSupportedSignContent, Result};
use serde::{Deserialize, Serialize};
use strum::Display;
use tea_runtime_codec::tapp::{Balance, ChannelId, ChannelItem};

#[derive(Debug, Serialize, Deserialize, Display)]
pub enum PaymentChannelTxn {
	OpenChannel {
		item: ChannelItem,
		auth_b64: String,
	},
	PayerEarlyTerminate {
		channel_id: ChannelId,
		auth_b64: String,
	},
	PayerTerminate {
		channel_id: ChannelId,
		auth_b64: String,
	},
	PayerRefill {
		channel_id: ChannelId,
		refill_amount: Balance,
		auth_b64: String,
	},
	UpdatePayment {
		channel_id: ChannelId,
		payment_update_sig: Vec<u8>,
		new_fund_remaining: Balance,
		close_channel: bool,
		auth_b64: String,
	},
}

impl PaymentChannelTxn {
	pub fn sign_content(&self) -> Result<Vec<u8>> {
		match self {
			PaymentChannelTxn::UpdatePayment {
				channel_id,
				new_fund_remaining,
				close_channel,
				..
			} => {
				let mut rtn = format!("{channel_id:?}").into_bytes();
				rtn.extend(new_fund_remaining.to_string().into_bytes());
				rtn.extend(close_channel.to_string().into_bytes());
				Ok(rtn)
			}
			_ => Err(NotSupportedSignContent.into()),
		}
	}
}
