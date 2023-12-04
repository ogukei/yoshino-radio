
use std::sync::Arc;
use cores::ipc::InvokeMessage;

use anyhow::Result;
use serde::Deserialize;

use crate::runtime_context::RuntimeContext;

#[derive(Deserialize)]
struct AnyEventCallbackBody {
    event: AnyEvent,
}

#[derive(Deserialize)]
struct AnyEvent {
    r#type: String,
    subtype: Option<String>,
    bot_id: Option<String>,
}

pub struct SlackEventMessageHandler {
    runtime_context: Arc<RuntimeContext>,
}

impl SlackEventMessageHandler {
    pub fn new(runtime_context: &Arc<RuntimeContext>) -> Arc<Self> {
        let runtime_context = Arc::clone(runtime_context);
        let handler: SlackEventMessageHandler = Self {
            runtime_context,
        };
        Arc::new(handler)
    }

    pub async fn process_event_callback(self: &Arc<Self>, event_type: String, body: String) -> Result<()> {
        match event_type.as_str() {
            "event_callback" => self.handle_slack_event_callback(event_type, body).await,
            _ => Ok(())
        }
    }

    async fn handle_slack_event_callback(&self, event_type: String, body: String) -> Result<()> {
        let any_event: AnyEventCallbackBody = serde_json::from_str(&body)?;
        let r#type = any_event.event.r#type.as_str();
        // ignore message updates
        let subtype = any_event.event.subtype
            .as_ref()
            .map(|v| v.as_str());
        // ignore bot's messages
        let bot_id = any_event.event.bot_id;
        if bot_id.is_some() {
            return Ok(())
        }
        match (r#type, subtype) {
            ("message", None) | ("message", Some("file_share")) => self.handle_slack_message(event_type, body).await,
            _ => Ok(()),
        }
    }

    async fn handle_slack_message(&self, event_type: String, body: String) -> Result<()>  {
        let channel_client = self.runtime_context.channel_client();
        let message = InvokeMessage {
            event_type,
            body,
        };
        channel_client.invoke(message).await?;
        Ok(())
    }
}
