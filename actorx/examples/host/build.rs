use std::{path::PathBuf, process::Command};

use tea_sdk::actorx::sign::sign_file;

fn main() {
	let mut manifest_dir =
		PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("'OUT_DIR' is not set"));
	let mut rebuild_dir = manifest_dir.clone();
	manifest_dir.push("../actor");
	manifest_dir = manifest_dir.canonicalize().unwrap();

	rebuild_dir.pop();
	rebuild_dir.pop();
	println!("cargo:rerun-if-changed={}", rebuild_dir.display());

	let mut wasm = PathBuf::from(std::env::var("OUT_DIR").expect("'OUT_DIR' is not set"));
	let out_dir = wasm.clone();
	wasm.push("wasm32-unknown-unknown");
	wasm.push("release");
	wasm.push("tea_actorx_examples_actor.wasm");
	_ = std::fs::remove_file(&wasm);

	Command::new("cargo")
		.arg("build")
		.arg("--release")
		.arg("--target")
		.arg("wasm32-unknown-unknown")
		.arg("-p")
		.arg("tea-actorx-examples-actor")
		.arg("--target-dir")
		.arg(out_dir)
		.spawn()
		.expect("msg")
		.wait()
		.unwrap();

	manifest_dir.push("manifest.yaml");
	let manifest = manifest_dir.clone();
	manifest_dir.pop();
	manifest_dir.push("key.pem");
	let priv_key = manifest_dir;
	sign_file(&wasm, manifest, priv_key).unwrap();
}
