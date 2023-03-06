pub const FROM_EMAIL_BOX: &str = "info@teaproject.org";
pub const SENDGRID_API: &str =
	"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";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendOtpForEmailLoginRequest {
	pub uuid: String,
	pub email: String,
	pub token_id: String,
	pub otp: Option<String>,
}

fn send_otp_with_sendgrid(email: &str, otp: &str) -> Result<third_api::ThirdAPiGeneralResponse> {
	let url = "https://api.sendgrid.com/v3/mail/send".to_string();

	let send_grid_api = SENDGRID_API.to_string();
	let header: Vec<(String, String)> = vec![
		(
			"Authorization".to_string(),
			format!("Bearer {}", send_grid_api),
		),
		("Content-Type".to_string(), "application/json".to_string()),
	];

	let payload = json!({
		"personalizations": [{
		"to": [{
		  "email": email,
		}],
		}],
	  "from": {
		"email": FROM_EMAIL_BOX
	  },
	"subject": "Your TEA Project Email Wallet Verification Code",
	"content": [
	  {
		"type": "text/html",
		"value": format!(r#"Your verification code is <b>{}</b> <br /><br />Please return to the TEA Project email wallet app and login using this verification code. You'll need to generate a new OTP code everytime you login to the email wallet. <br /><br />The email wallet is designed for users who are new to the TEA Project and don't have a MetaMask wallet setup yet. The email wallet allows you to interact with TApps in the TEA ecosystem but you will not be able to withdraw to the Ethereum chain through the email wallet app (you'll need to setup MetaMask if you want to start logging in without generating verification codes). By following the link, you'll be prompted to enter your verification code and to login to your email wallet (or create a new email wallet if it's your first time using the app). Please visit our <a href="https://t.me/teaprojectorg">Telegram</a> or <a href="https://discord.com/invite/nvtaneQgGb">Discord</a> if you have any questions."#, otp),
	  }
	  ]
	  });

	let buf = third_api::CryptPostRequest {
		url: serialize(&url)?,
		header: serialize(&header)?,
		payload: serde_json::to_vec(&payload)?,
	};

	let res = third_api::ThirdAPiGeneralResponse::decode(
		wascc_call(
			tea_runtime_codec::THIRD_API_CAPABILITY_ID,
			"crypt_post",
			&encode_protobuf(buf)?,
		)?
		.as_slice(),
	)?;

	Ok(res)
}
