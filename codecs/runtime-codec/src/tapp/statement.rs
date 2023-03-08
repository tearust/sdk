use crate::tapp::{
	error::{Error, StatementTypeParse},
	Account, Balance, TokenId,
};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Statement {
	pub account: Account,
	pub gross_amount: Balance,
	pub statement_type: StatementType,
	pub token_id: TokenId,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum StatementType {
	Incoming,
	Outcoming,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum StateType {
	Tea,
	TeaReserved,
	Bonding,
	BondingReserved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedStatement {
	pub statement: Statement,
	pub state_type: StateType,
}

impl Statement {
	pub fn new(
		account: Account,
		gross_amount: Balance,
		statement_type: StatementType,
		token_id: TokenId,
	) -> Self {
		Statement {
			account,
			gross_amount,
			statement_type,
			token_id,
		}
	}
}

impl TypedStatement {
	pub fn new(statement: Statement, state_type: StateType) -> Self {
		TypedStatement {
			statement,
			state_type,
		}
	}
}

impl Display for StatementType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			StatementType::Incoming => write!(f, "Incoming"),
			StatementType::Outcoming => write!(f, "Outcoming"),
		}
	}
}

impl FromStr for StatementType {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Incoming" => Ok(StatementType::Incoming),
			"Outcoming" => Ok(StatementType::Outcoming),
			_ => Err(StatementTypeParse(s.to_string()).into()),
		}
	}
}

impl Display for StateType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			StateType::Tea => write!(f, "Tea"),
			StateType::TeaReserved => write!(f, "TeaReserved"),
			StateType::Bonding => write!(f, "Bonding"),
			StateType::BondingReserved => write!(f, "BondingReserved"),
		}
	}
}

impl FromStr for StateType {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Tea" => Ok(StateType::Tea),
			"TeaReserved" => Ok(StateType::TeaReserved),
			"Bonding" => Ok(StateType::Bonding),
			"BondingReserved" => Ok(StateType::BondingReserved),
			_ => Err(StatementTypeParse(s.to_string()).into()),
		}
	}
}
