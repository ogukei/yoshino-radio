
use std::sync::Arc;

use anyhow::Result;
use cores::ipc::InvokeMessage;
use serde::Deserialize;
use tracing::info;

use crate::{slack_client::SlackClient};

use serde_json;

#[derive(Deserialize)]
struct AnyEventCallbackBody {
    event: AnyEvent,
}

#[derive(Deserialize)]
struct AnyEvent {
    r#type: String,
}


#[derive(Deserialize)]
struct MessageEventCallbackBody {
    event: MessageEvent,
}

#[derive(Deserialize)]
struct MessageEvent {
    bot_id: Option<String>,
    text: String,
    channel: String,
}

pub struct MessageHandle {
}

impl MessageHandle {
    pub fn new() -> Arc<Self> {
        let server = Self {
        };
        Arc::new(server)
    }

    // https://api.slack.com/events/message.im
    pub async fn handle_message(&self, message: InvokeMessage) -> Result<()> {
        info!("worker received {}, {}", message.event_type, message.body);
        match message.event_type.as_str() {
            "event_callback" => self.handle_slack_event_callback(&message.body).await,
            _ => Ok(())
        }
    }

    pub async fn handle_slack_event_callback(&self, body: &str) -> Result<()> {
        let any_event: AnyEventCallbackBody = serde_json::from_str(body)?;
        match any_event.event.r#type.as_str() {
            "message" => self.handle_slack_message(body).await,
            _ => Ok(()),
        }
    }

    pub async fn handle_slack_message(&self, body: &str) -> Result<()> {
        let body: MessageEventCallbackBody = serde_json::from_str(body)?;
        let message_event = body.event;
        if message_event.bot_id.is_some() {
            return Ok(())
        }
        let client = SlackClient::new().await?;
        client.send(message_event.channel, message_event.text + "!").await?;
        Ok(())
    }
}
