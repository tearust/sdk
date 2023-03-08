use crate::enclave::actors::http::RequestExt;
use serde::Deserialize;
use serde::Serialize;

use crate::client::error::Result;
use crate::client::help;
use crate::client::request;

//const TWITTER_OAUTH_KEY: &str = "AAAAAAAAAAAAAAAAAAAAAF4gTwEAAAAAedgLBTRn%2Bp78NXs2n12t7xhbcl8%3DRE8ZxY6KcGFaUKYa1F5oWgf6pE0rBC8Us8A3hiIdEUwx4rUF8f";
//const TWITTER_CANDIDATE_KEY: &str = "twitter_candidate_key";

#[derive(Debug, Serialize, Deserialize)]
struct ReferencedTweets {
	pub r#type: String,
	pub id: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct RetweetCheckResult {
	pub id: String,
	pub text: String,
	pub referenced_tweets: Vec<ReferencedTweets>,
}
#[derive(Debug, Serialize, Deserialize)]
struct WrapRetweetCheckResult {
	pub data: Option<RetweetCheckResult>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwitterRetweetForUserRequest {
	pub uuid: String,
	pub from: String,

	pub target: String,
	pub tweet_id: String,
	pub token_id: String,
}

pub async fn retweet_check_for_twitter(_payload: Vec<u8>, _from_actor: String) -> Result<Vec<u8>> {
	// let rs = call(
	// 	tea_actorx_core::RegId::Static(tea_system_actors::http::NAME).inst(0),
	// 	tea_system_actors::http::HyperRequest("a".to_string(), vec![]),
	// )
	// .await?;
	info!("@@ 11111");
	let builder = request::http::Request::builder()
		.method("GET")
		.uri("https://api.twitter.com/2/tweets/1608596220297240578")
		.header("Authorization", "Bearer AAAAAAAAAAAAAAAAAAAAAF4gTwEAAAAAedgLBTRn%2Bp78NXs2n12t7xhbcl8%3DRE8ZxY6KcGFaUKYa1F5oWgf6pE0rBC8Us8A3hiIdEUwx4rUF8f");
	info!("@@ 3333 => {:?}", builder);
	let result = builder.request::<serde_json::Value>().await?;
	info!("@@ 2222 => {:?}", result);
	// check_retweet_id(&from_actor, &req.token_id, res)?;

	// let txn = TappstoreTxn::TwitterRetweetFaucetForAccount {
	// 	twitter_id: req.tweet_id.to_string(),
	// 	target: actor_util::str_to_h160(&req.target)?,
	// 	token_id: req.token_id.to_string(),
	// };
	// let txn_bytes: Vec<u8> = tea_codec::serialize(&txn)?;

	// send_txn(
	// 	from_actor,
	// 	"retweet_check_for_twitter",
	// 	&req.uuid,
	// 	tea_codec::serialize(&req)?,
	// 	txn_bytes,
	// 	vec![],
	// 	tea_runtime_codec::ACTOR_PUBKEY_TAPPSTORE,
	// )?;

	help::result_ok()
}
