#[cfg(all(feature = "host", not(rust_analyzer)))]
fn main() {
	use std::{env, fs, path::Path, process::Command};
	let profile = env::var("PROFILE").expect("PROFILE is not set");
	let profile_cmd = if profile == "debug" { "dev" } else { &profile };

	let out_dir = env::var("OUT_DIR").expect("OUT_DIR is not set");

	#[allow(unused_mut)]
	let mut features = vec!["worker"];

	macro_rules! pass_features {
		($($f:literal),*) => {$(
			#[cfg(feature = $f)]
			features.push($f);
		)*};
	}

	pass_features!("nitro", "metering", "backtrace", "verbose_log");
	#[cfg(feature = "metering")]
	features.push("wasmer-middlewares");
	#[cfg(feature = "__test")]
	features.push("__test");

	let target = env::var("TARGET").expect("TARGET is not set");

	Command::new("cargo")
		.arg("build")
		.arg("--target")
		.arg(&target)
		.arg("--profile")
		.arg(profile_cmd)
		.arg("-p")
		.arg("tea-actorx")
		.arg("--bin")
		.arg("worker")
		.arg("--no-default-features")
		.arg("--features")
		.arg(features.join(","))
		.arg("--target-dir")
		.arg(&out_dir)
		.current_dir(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set"))
		.spawn()
		.expect("msg")
		.wait()
		.unwrap();

	let mut bin_path = Path::new(&out_dir).to_path_buf();
	bin_path.push(target);
	bin_path.push(profile);
	bin_path.push("worker");
	fs::rename(bin_path, Path::new(&out_dir).join("worker")).unwrap();
}

#[cfg(any(not(feature = "host"), rust_analyzer))]
fn main() {}
