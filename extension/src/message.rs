
use std::sync::Arc;

use cores::ipc::ChannelMessage;

use anyhow::Result;
use serde::Deserialize;
use tracing::info;

use crate::{extension_context::ExtensionContext, slack_client::SlackClient};

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
    text: String,
    channel: String,
}


pub struct MessageHandle {
    extension_context: Arc<ExtensionContext>,
}

impl MessageHandle {
    pub fn new(extension_context: &Arc<ExtensionContext>) -> Arc<Self> {
        let extension_context = Arc::clone(extension_context);
        let server = Self {
            extension_context,
        };
        Arc::new(server)
    }

    // https://api.slack.com/events/message.im
    pub async fn handle_message(&self, message: ChannelMessage) -> Result<()> {
        let ChannelMessage::SlackEvent(slack_event) = message;
        info!("extension received {}, {}", slack_event.event_type, slack_event.body);
        match slack_event.event_type.as_str() {
            "event_callback" => self.handle_slack_event_callback(&slack_event.body).await,
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
        let client = SlackClient::new(&self.extension_context).await?;
        client.send(message_event.channel, message_event.text + "!").await?;
        Ok(())
    }
}
