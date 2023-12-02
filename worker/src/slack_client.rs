
use std::{sync::Arc, env};
use anyhow::Result;
use reqwest;
use serde::{Deserialize, Serialize};
use tracing::info;


#[derive(Serialize)]
struct RequestBody {
    channel: String,
    text: String,
}

pub struct SlackClient {
}

// @see https://api.slack.com/rtm#sending_messages

impl SlackClient {
    pub async fn new() -> Result<Arc<Self>> {
        let this = Self {
        };
        let this = Arc::new(this);
        Ok(this)
    }
    
    pub async fn send(&self, channel: String, text: String) -> Result<()> {
        let client_token = env::var("SLACK_CLIENT_TOKEN")?;
        let client = reqwest::Client::new();
        let request_body = RequestBody {
            channel,
            text,
        };
        let response = client.post("https://slack.com/api/chat.postMessage")
            .header("Content-type", "application/json")
            .header("Authorization", ["Bearer", &client_token].join(" "))
            .json(&request_body)
            .send()
            .await?;
        let text = response.text().await?;
        info!("slack response {:?}", text);
        //
        Ok(())
    }
}
