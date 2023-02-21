#![feature(iterator_try_collect)]

use tea_codec::ResultExt;
use tokio::fs;

use clap::Parser;
use error::Result;
use tea_actorx_signer::*;

#[derive(Debug, Parser)]
struct Cli {
	wasm: String,

	#[arg(short, long)]
	id: String,

	#[arg(short, long)]
	key: String,

	#[arg(short, long)]
	access: Vec<String>,

	#[arg(short, long)]
	token_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
	let cli = Cli::parse();
	let wasm_path = cli.wasm;
	let (wasm, key, id) = tokio::join!(
		fs::read(wasm_path.as_str()),
		fs::read(cli.key),
		fs::read(cli.id)
	);
	let (mut wasm, key, id) = (wasm?, key?, id?);
	let payload = Metadata {
		id,
		signer: key,
		claims: cli
			.access
			.into_iter()
			.map(|input| {
				Ok(Claim::ActorAccess(
					if let [b'#', input @ ..] = input.as_bytes() {
						base64::decode(input)?
					} else {
						input.into_bytes()
					},
				)) as Result<_>
			})
			.chain(Some(cli.token_id.parse().map(Claim::TokenId).err_into()))
			.try_collect()?,
	};

	sign(&mut wasm, payload)?;

	fs::write(wasm_path, wasm).await?;

	Ok(())
}
