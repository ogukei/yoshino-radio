
use std::{sync::Arc, env};
use anyhow::Result;
use reqwest::{self, Client};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Serialize)]
struct PostRequestBody {
    channel: String,
    text: String,
    thread_ts: Option<String>,
}

#[derive(Deserialize)]
struct PostResponseBody {
    channel: String,
    ts: String,
}

pub struct PostResult {
    pub channel: String,
    pub ts: String,
}

#[derive(Serialize)]
struct UpdateRequestBody {
    channel: String,
    ts: String,
    text: String,
}

#[derive(Deserialize)]
struct UpdateResponseBody {
    channel: String,
    ts: String,
}

pub struct SlackClient {
    client: Client,
}

// https://api.slack.com/messaging/sending
impl SlackClient {
    pub async fn new() -> Result<Arc<Self>> {
        let client = reqwest::Client::new();
        let this = Self {
            client,
        };
        let this = Arc::new(this);
        Ok(this)
    }

    // https://api.slack.com/methods/chat.postMessage
    pub async fn post(&self, channel: String, thread_ts: Option<String>, text: String) -> Result<PostResult> {
        let client_token = env::var("SLACK_CLIENT_TOKEN")?;
        let request_body = PostRequestBody {
            channel,
            text,
            thread_ts,
        };
        let response = self.client.post("https://slack.com/api/chat.postMessage")
            .header("Content-type", "application/json; charset=utf-8")
            .header("Authorization", ["Bearer", &client_token].join(" "))
            .json(&request_body)
            .send()
            .await?;
        let text = response.text().await?;
        info!("slack chat.postMessage response {:?}", text);
        let response: PostResponseBody = serde_json::from_str(&text)?;
        let result = PostResult {
            channel: response.channel,
            ts: response.ts,
        };
        Ok(result)
    }

    // https://api.slack.com/methods/chat.update
    pub async fn update(&self, channel: String, ts: String, text: String) -> Result<()> {
        let client_token = env::var("SLACK_CLIENT_TOKEN")?;
        let request_body = UpdateRequestBody {
            channel,
            ts,
            text,
        };
        let response = self.client.post("https://slack.com/api/chat.update")
            .header("Content-type", "application/json; charset=utf-8")
            .header("Authorization", ["Bearer", &client_token].join(" "))
            .json(&request_body)
            .send()
            .await?;
        let text = response.text().await?;
        info!("slack chat.update response {:?}", text);
        let _response: UpdateResponseBody = serde_json::from_str(&text)?;
        Ok(())
    }
}
