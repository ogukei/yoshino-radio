
// SLACK_SIGNING_SECRET
use std::env;

use sha2::Sha256;
use hmac::{Hmac, Mac};
use hex;

use lambda_http::{Request, Body};
use std::time::{SystemTime, Duration};
use anyhow::{Context, Result, bail};

type HmacSha256 = Hmac<Sha256>;

// https://api.slack.com/authentication/verifying-requests-from-slack
pub fn verify_slack_request(request: &Request) -> Result<()> {
    let headers = request.headers();
    let Body::Text(body_text) = request.body() else {
        bail!("no body");
    };
    let slack_signature = headers.get("X-Slack-Signature")
        .context("X-Slack-Signature is empty")?
        .to_str()?;
    let slack_timestamp = headers.get("X-Slack-Request-Timestamp")
        .context("X-Slack-Request-Timestamp is empty")?
        .to_str()?;
    let slack_timestamp_time: u64 = slack_timestamp.parse()?;
    let slack_timestamp_time = Duration::from_secs(slack_timestamp_time);
    let now = SystemTime::now();
    let now = now.duration_since(SystemTime::UNIX_EPOCH)?;
    let delta = now - slack_timestamp_time;
    if delta > Duration::from_secs(5 * 60) {
        bail!("The request timestamp is more than five minutes from local time");
    }
    let verification_result = verify_signature(slack_timestamp, body_text, slack_signature)?;
    if verification_result {
        Ok(())
    } else {
        bail!("verification failed")
    }
}

// https://api.slack.com/authentication/verifying-requests-from-slack#making__validating-a-request
fn verify_signature(timestamp: &str, body: &str, signature_actual: &str) -> Result<bool> {
    let signing_secret = env::var("SLACK_SIGNING_SECRET")?;
    let mut mac = HmacSha256::new_from_slice(signing_secret.as_bytes())?;
    let message = ["v0", timestamp, body].join(":");
    mac.update(message.as_bytes());
    let mac = mac.finalize();
    let signature_expected = mac.into_bytes();
    let signature_expected = hex::encode(&signature_expected);
    let signature_expected = ["v0=", signature_expected.as_str()].join("");
    Ok(signature_actual == signature_expected)
}
