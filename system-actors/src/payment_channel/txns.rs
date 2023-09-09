use super::error::{NotSupportedSignContent, Result};
use serde::{Deserialize, Serialize};
use strum::Display;
use tea_runtime_codec::tapp::{Account, Balance, ChannelId, ChannelItem, TimestampShort};

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
	Terminate {
		channel_id: ChannelId,
		auth_b64: String,
		from_user: Account,
	},
	PayerRefill {
		channel_id: ChannelId,
		refill_amount: Balance,
		auth_b64: String,
	},
	UpdatePayment {
		channel_id: ChannelId,
		payment_update_sig: String,
		new_fund_remaining: Balance,
		close_channel: bool,
	},
	ScheduledGasPayment {
		timestamp: TimestampShort,
	},
}

impl PaymentChannelTxn {
	pub fn sign_content(&self) -> Result<String> {
		match self {
			PaymentChannelTxn::UpdatePayment {
				new_fund_remaining, ..
			} => {
				let rtn = new_fund_remaining.to_string();
				Ok(rtn)
			}
			_ => Err(NotSupportedSignContent.into()),
		}
	}
}
