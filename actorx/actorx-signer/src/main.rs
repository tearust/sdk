#![feature(iterator_try_collect)]

mod config;

use std::{env, path::PathBuf};

use config::Manifest;
use tokio::fs;

use clap::Parser;
use error::Result;
use tea_actorx_signer::*;

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
		wasm: wasm_path,
		manifest,
		key,
	} = Cli::parse();
	let manifest = manifest
		.map(Into::into)
		.unwrap_or_else(current_dir_file("manifest.yaml"));
	let key = key
		.map(Into::into)
		.unwrap_or_else(current_dir_file("key.pem"));
	let (wasm, key, manifest) =
		tokio::join!(fs::read(&wasm_path), fs::read(key), fs::read(manifest));
	let (mut wasm, key, manifest) = (wasm?, key?, manifest?);

	if verify(&wasm).is_ok() {
		return Ok(());
	}

	let manifest = serde_yaml::from_slice::<Manifest>(&manifest)?;
	sign(&mut wasm, manifest.into_metadata(key)?)?;

	fs::write(wasm_path, wasm).await?;

	Ok(())
}

fn current_dir_file(name: &str) -> impl Fn() -> PathBuf + '_ {
	move || {
		let mut path = env::current_dir().unwrap();
		path.push(name);
		path
	}
}
