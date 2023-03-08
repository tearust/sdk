#[cfg(feature = "vmh")]
use std::{
	env, fs,
	path::{Path, PathBuf},
};

#[cfg(feature = "vmh")]
fn main() {
	let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("'CARGO_MANIFEST_DIR' is not set");
	let include_path = Path::new(&manifest_dir).join("proto");

	let proto_files: Vec<PathBuf> = fs::read_dir(&include_path)
		.expect("failed to find include path")
		.map(|v| v.unwrap().path())
		.collect();

	println!("cargo:rerun-if-changed={}", include_path.to_str().unwrap());

	println!(
		"rebuild protobuf files (manifest dir: {}, include path: {:?}, proto files: {:?})...",
		manifest_dir,
		include_path.to_str(),
		proto_files
	);

	let mut out_path = PathBuf::from(env::var("OUT_DIR").expect("'OUT_DIR' is not set"));
	out_path.push("structs_proto");
	if !out_path.exists() {
		std::fs::create_dir_all(&out_path).expect("create proto out directory failed");
	}

	prost_build::Config::new()
		.out_dir(out_path)
		.compile_protos(&proto_files, &[include_path])
		.unwrap()
}

#[cfg(not(feature = "vmh"))]
fn main() {}
