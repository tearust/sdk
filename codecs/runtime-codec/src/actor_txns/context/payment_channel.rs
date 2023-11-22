use super::{IsBalanceRelated, PaymentChannelContext, Result};
use crate::tapp::{Balance, ChannelId, ChannelItem};
use serde::{Deserialize, Serialize};
use sha2::digest::Update;
use std::collections::{HashMap, HashSet};
use tea_sdk::serialize;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaymentChannelContextImpl {
	new_channels: HashMap<ChannelId, ChannelItem>,
	update_payments: HashMap<ChannelId, (Balance, bool)>,
	early_terminate: HashSet<ChannelId>,
	terminate: HashSet<ChannelId>,
	payee_terminate: HashSet<ChannelId>,
	payer_refills: HashMap<ChannelId, Balance>,
}

impl IsBalanceRelated for PaymentChannelContextImpl {
	fn is_balance_related(&self) -> bool {
		!self.new_channels.is_empty()
			|| !self.update_payments.is_empty()
			|| !self.payer_refills.is_empty()
			|| !self.early_terminate.is_empty()
			|| !self.terminate.is_empty()
			|| !self.payee_terminate.is_empty()
	}
}

impl PaymentChannelContext for PaymentChannelContextImpl {
	fn create_channel(&mut self, item: ChannelItem) {
		self.new_channels.insert(item.channel_id.clone(), item);
	}

	fn add_update_payment(&mut self, channel_id: ChannelId, remaining: Balance, close: bool) {
		self.update_payments.insert(channel_id, (remaining, close));
	}

	fn add_payer_early_terminate(&mut self, channel_id: ChannelId) {
		self.early_terminate.insert(channel_id);
	}

	fn add_payer_terminate(&mut self, channel_id: ChannelId) {
		self.terminate.insert(channel_id);
	}

	fn add_payee_terminate(&mut self, channel_id: ChannelId) {
		self.payee_terminate.insert(channel_id);
	}

	fn add_payer_refill(&mut self, channel_id: ChannelId, refill_amount: Balance) {
		self.payer_refills.insert(channel_id, refill_amount);
	}

	fn get_new_channels(&self) -> &HashMap<ChannelId, ChannelItem> {
		&self.new_channels
	}

	fn get_update_payments(&self) -> &HashMap<ChannelId, (Balance, bool)> {
		&self.update_payments
	}

	fn get_payer_early_terminate(&self) -> &HashSet<ChannelId> {
		&self.early_terminate
	}

	fn get_payer_terminate(&self) -> &HashSet<ChannelId> {
		&self.terminate
	}

	fn get_payee_terminate(&self) -> &HashSet<ChannelId> {
		&self.payee_terminate
	}

	fn get_payer_refills(&self) -> &HashMap<ChannelId, Balance> {
		&self.payer_refills
	}
}

impl PaymentChannelContextImpl {
	pub fn hash(&self, hasher: &mut sha2::Sha256) -> Result<()> {
		let mut new_channels = self.new_channels.iter().collect::<Vec<_>>();
		new_channels.sort_by(|a, b| a.0.cmp(&b.0));
		for (channel_id, item) in new_channels {
			hasher.update(channel_id.as_ref());
			hasher.update(&serialize(item)?);
		}

		let mut update_payments = self.update_payments.iter().collect::<Vec<_>>();
		update_payments.sort_by(|a, b| a.0.cmp(&b.0));
		for (channel_id, v) in update_payments {
			hasher.update(channel_id.as_ref());
			hasher.update(&serialize(v)?);
		}

		let mut payer_refills = self.payer_refills.iter().collect::<Vec<_>>();
		payer_refills.sort_by(|a, b| a.0.cmp(&b.0));
		for (channel_id, v) in payer_refills {
			hasher.update(channel_id.as_ref());
			hasher.update(&serialize(v)?);
		}

		self.hash_channels(&self.early_terminate, hasher)?;
		self.hash_channels(&self.terminate, hasher)?;
		self.hash_channels(&self.payee_terminate, hasher)?;
		Ok(())
	}

	fn hash_channels(&self, set: &HashSet<ChannelId>, hasher: &mut sha2::Sha256) -> Result<()> {
		let mut token_set = set.iter().collect::<Vec<_>>();
		token_set.sort();
		for acc in token_set {
			hasher.update(acc.as_ref());
		}
		Ok(())
	}
}
