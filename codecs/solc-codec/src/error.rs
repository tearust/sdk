use std::str::FromStr;

use ethereum_types::H160;
use runtime_codec::error::RuntimeCodec;
use tea_sdk::{define_scope, errorx::Global};

type FromHexError = <H160 as FromStr>::Err;
define_scope! {
	SolcCodec: RuntimeCodec {
		FromHexError => @Global::HexDecode, @Display, @Debug;
		Layer1;
	}
}
