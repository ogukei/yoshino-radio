
use std::sync::Arc;
use crate::extension_context::ExtensionContext;
use anyhow::Result;
use reqwest;
use serde::{Deserialize, Serialize};
use tracing::info;


#[derive(Serialize)]
struct RequestBody {
    channel: String,
    text: String,
}

#[derive(Deserialize)]
struct ResponseBody {
    ok: bool,
}

pub struct SlackClient {
    extension_context: Arc<ExtensionContext>,
}

// @see https://api.slack.com/rtm#sending_messages

impl SlackClient {
    pub async fn new(extension_context: &Arc<ExtensionContext>) -> Result<Arc<Self>> {
        let extension_context = Arc::clone(extension_context);
        let this = Self {
            extension_context,
        };
        let this = Arc::new(this);
        Ok(this)
    }
    
    pub async fn send(&self, channel: String, text: String) -> Result<()> {
        //
        info!("SlackClient getting client token");
        let client_token = std::env::var("SLACK_CLIENT_TOKEN")?;
        info!("SlackClient getting client token ok");
        let client = reqwest::Client::new();
        let request_body = RequestBody {
            channel,
            text,
        };
        info!("SlackClient posting...");
        let response = client.post("https://slack.com/api/chat.postMessage")
            .header("Content-type", "application/json")
            .header("Authorization", client_token)
            .json(&request_body)
            .send()
            .await?;
        info!("SlackClient post complete");
        let response_body = response.json::<ResponseBody>().await?;
        info!("extension got ok {}", response_body.ok);
        //
        Ok(())
    }
}
