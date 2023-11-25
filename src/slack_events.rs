
use lambda_http::{Body, Request, Response};
use serde_json;

use serde::Deserialize;
use anyhow::{Result, bail};

// https://api.slack.com/apis/connections/events-api#handshake
#[derive(Deserialize, Debug)]
struct CommonEventContent {
    r#type: String,
}

pub async fn handle_verified_slack_events(event: Request) -> Result<Response<Body>> {
    let Body::Text(body) = event.body() else {
        bail!("no body");
    };
    let common_content: CommonEventContent = serde_json::from_str(body)?;
    match common_content.r#type.as_str() {
        "url_verification" => url_verification(event),
        _ => {
            let response = Response::builder()
                .status(403)
                .header("content-type", "text/plain")
                .body("forbidden".into())
                .map_err(Box::new)?;
            Ok(response)
        }
    }
}

// https://api.slack.com/apis/connections/events-api#handshake
#[derive(Deserialize, Debug)]
struct Handshake {
    challenge: String,
}

fn url_verification(event: Request) -> Result<Response<Body>> {
    let Body::Text(body) = event.body() else {
        bail!("no body");
    };
    let handshake: Handshake = serde_json::from_str(body)?;
    let response = Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body(handshake.challenge.into())
        .map_err(Box::new)?;
    Ok(response)
}
