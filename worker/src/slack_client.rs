
use std::{sync::Arc, env, os::unix::thread};
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
    message: PostResponseMessage,
}

#[derive(Deserialize)]
struct PostResponseMessage {
    thread_ts: Option<String>,
}

pub struct PostResult {
    pub channel: String,
    pub ts: String,
    pub thread_ts: Option<String>,
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

#[derive(Serialize)]
struct RepliesRequestBody {
    channel: String,
    ts: String,
}

#[derive(Deserialize)]
struct RepliesResponseBody {
    messages: Vec<RepliesMessage>,
}

#[derive(Debug)]
pub struct RepliesResult {
    pub messages: Vec<RepliesMessage>,
}

#[derive(Deserialize, Debug)]
pub struct RepliesMessage {
    pub r#type: String,
    pub ts: String,
    pub text: String,
    pub thread_ts: String,
    pub bot_id: Option<String>,
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
    pub async fn post(&self, channel: &str, thread_ts: Option<&str>, text: String) -> Result<PostResult> {
        let client_token = env::var("SLACK_CLIENT_TOKEN")?;
        let request_body = PostRequestBody {
            channel: channel.into(),
            text: text.into(),
            thread_ts: thread_ts.map(|v| v.into()),
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
            thread_ts: response.message.thread_ts,
        };
        Ok(result)
    }

    // https://api.slack.com/methods/chat.update
    pub async fn update(&self, channel: &str, ts: &str, text: String) -> Result<()> {
        let client_token = env::var("SLACK_CLIENT_TOKEN")?;
        let request_body = UpdateRequestBody {
            channel: channel.into(),
            ts: ts.into(),
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

    // https://api.slack.com/methods/conversations.replies
    pub async fn replies(&self, channel: &str, ts: &str) -> Result<RepliesResult> {
        let client_token = env::var("SLACK_CLIENT_TOKEN")?;
        let response = self.client.get("https://slack.com/api/conversations.replies")
            .header("Content-type", "application/x-www-form-urlencoded; charset=utf-8")
            .header("Authorization", ["Bearer", &client_token].join(" "))
            .query(&[("channel", channel), ("ts", &ts)])
            .send()
            .await?;
        let text = response.text().await?;
        info!("slack conversations.replies response {:?}", text);
        let response: RepliesResponseBody = serde_json::from_str(&text)?;
        let result = RepliesResult {
            messages: response.messages,
        };
        Ok(result)
        }
}
