use std::str::FromStr;

use crate::runtime::error::RuntimeCodec;
use ethereum_types::H160;
use tea_sdk::{define_scope, errorx::Global};

type FromHexError = <H160 as FromStr>::Err;
define_scope! {
	SolcCodec: RuntimeCodec {
		FromHexError => @Global::HexDecode, @Display, @Debug;
		Layer1;
	}
}
