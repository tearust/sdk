#[cfg(feature = "host")]
fn main() {
	use std::{env, fs, path::Path, process::Command};
	let profile = env::var("PROFILE").expect("PROFILE is not set");
	let profile_cmd = if profile == "debug" { "dev" } else { &profile };

	let out_dir = env::var("OUT_DIR").expect("OUT_DIR is not set");

	Command::new("cargo")
		.arg("build")
		.arg("--profile")
		.arg(profile_cmd)
		.arg("-p")
		.arg("tea-actorx2")
		.arg("--bin")
		.arg("worker")
		.arg("--features")
		.arg("worker")
		.arg("--target-dir")
		.arg(&out_dir)
		.current_dir(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set"))
		.spawn()
		.expect("msg")
		.wait()
		.unwrap();

	let mut bin_path = Path::new(&out_dir).join(profile);
	bin_path.push("worker");
	fs::rename(bin_path, Path::new(&out_dir).join("worker")).unwrap();
}

#[cfg(not(feature = "host"))]
fn main() {}
