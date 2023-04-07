use std::path::PathBuf;

use clap::Parser;
use tea_actorx2::sign::{error::Result, sign_file};

#[derive(Debug, Parser)]
struct Cli {
	wasm: String,
	#[arg(short, long)]
	manifest: Option<String>,
	key: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
	let Cli {
		wasm,
		manifest,
		key,
	} = Cli::parse();
	let manifest = manifest
		.map(Into::into)
		.map(Ok)
		.unwrap_or_else(current_dir_file("manifest.yaml"))?;
	let priv_key = key
		.map(Into::into)
		.map(Ok)
		.unwrap_or_else(current_dir_file("key.pem"))?;

	sign_file(wasm, manifest, priv_key)
}

fn current_dir_file(name: &str) -> impl Fn() -> Result<PathBuf> + '_ {
	move || {
		let mut path = std::env::current_dir()?;
		path.push(name);
		Ok(path)
	}
}
