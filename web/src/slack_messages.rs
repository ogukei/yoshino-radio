
use std::sync::Arc;
use cores::ipc::{ChannelMessage, SlackEventMessage};

use anyhow::Result;

use crate::runtime_context::RuntimeContext;

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
        let channel_client = self.runtime_context.channel_client();
        let slack_event = SlackEventMessage {
            event_type,
            body,
        };
        let message = ChannelMessage::SlackEvent(slack_event);
        channel_client.invoke(message).await?;
        Ok(())
    }
}