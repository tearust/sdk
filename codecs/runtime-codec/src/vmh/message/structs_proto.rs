#![allow(clippy::all)] // disable clippy for auto-generated codes

macro_rules! import_proto {
	[$($name:ident),*] => {
		$(pub mod $name {
			include!(concat!(
				env!("OUT_DIR"),
				"/structs_proto/",
				stringify!($name),
				".rs"
			));
		})*
	};
}

import_proto![
	crypto, env, intercom, ipfs, kvp, layer1, libp2p, orbitdb, p2p, persist, pinner, ra, raft,
	receipt, replica, report, rpc, tappstore, third_api, tokenstate, tpm, vmh
];
