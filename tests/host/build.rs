use std::{path::PathBuf, process::Command};

use tea_sdk::actorx::sign::sign_file;

fn main() {
	let manifest_dir =
		PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("'OUT_DIR' is not set"));
	let mut rebuild_dir = manifest_dir.clone();

	rebuild_dir.pop();
	rebuild_dir.pop();
	println!("cargo:rerun-if-changed={}", rebuild_dir.display());

	prepare_wasm_actor(
		"wasm-a",
		"wasm-a-actor",
		"wasm_a_actor.wasm",
		manifest_dir.clone(),
	);
	prepare_wasm_actor("wasm-b", "wasm-b-actor", "wasm_b_actor.wasm", manifest_dir);
}

fn prepare_wasm_actor(sub_dir: &str, pkg_name: &str, actor_name: &str, mut manifest_dir: PathBuf) {
	manifest_dir.push(format!("../{sub_dir}"));
	manifest_dir = manifest_dir.canonicalize().unwrap();

	let mut wasm = PathBuf::from(std::env::var("OUT_DIR").expect("'OUT_DIR' is not set"));
	let out_dir = wasm.clone();
	wasm.push("wasm32-unknown-unknown");
	wasm.push("release");
	wasm.push(actor_name);
	_ = std::fs::remove_file(&wasm);

	Command::new("cargo")
		.arg("build")
		.arg("--release")
		.arg("--target")
		.arg("wasm32-unknown-unknown")
		.arg("-p")
		.arg(pkg_name)
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
	println!(
		"manifest: {:?}, wasm: {:?}, priv_key: {:?}",
		manifest, wasm, priv_key
	);
	sign_file(&wasm, manifest, priv_key).unwrap();
}
