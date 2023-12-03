
use std::sync::Arc;

use anyhow::Result;
use cores::ipc::InvokeMessage;
use serde::Deserialize;
use tracing::info;
use futures_util::StreamExt;

use crate::{slack_client::SlackClient, openai_client::{OpenAIClient, CompletionsRequestMessage, CompletionsMessageChunk}};

use serde_json;

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

#[derive(Deserialize)]
struct MessageEventCallbackBody {
    event: MessageEvent,
}

#[derive(Deserialize)]
struct MessageEvent {
    text: String,
    channel: String,
    ts: String,
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

    async fn handle_slack_event_callback(&self, body: &str) -> Result<()> {
        let any_event: AnyEventCallbackBody = serde_json::from_str(&body)?;
        let r#type = any_event.event.r#type.as_str();
        // ignore message updates
        let subtype = any_event.event.subtype;
        // ignore bot's messages
        let bot_id = any_event.event.bot_id;
        match (r#type, subtype, bot_id) {
            ("message", None, None) => self.handle_slack_message(body).await,
            _ => Ok(()),
        }
    }

    async fn handle_slack_message(&self, body: &str) -> Result<()> {
        let body: MessageEventCallbackBody = serde_json::from_str(body)?;
        let message_event = body.event;
        let text = message_event.text.clone();
        let client = SlackClient::new().await?;
        let result = client.post(
            message_event.channel, 
            Some(message_event.ts),
             format!("Hi! `[Processing {}...]`", message_event.text)).await?;
        let openai_client = OpenAIClient::new().await?;
        let messages = vec![
            CompletionsRequestMessage {
                role: "user".into(),
                content: text,
            }
        ];
        let mut completions = String::new();
        let mut stream = openai_client.completions(messages).await?;
        while let Some(item) = stream.next().await {
            let Ok(chunks) = item else { continue };
            let chunks: Vec<CompletionsMessageChunk> = chunks;
            for chunk in chunks {
                for choise in chunk.choices {
                    if let Some(content) = choise.delta.content {
                        completions += &content;
                    }
                }
            }
        }
        client.update(result.channel, result.ts, completions).await?;
        Ok(())
    }
}
