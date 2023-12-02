
use std::sync::Arc;

use lambda_http::{Body, Request, Response};
use serde_json;

use serde::Deserialize;
use anyhow::{Result, bail};

use crate::{slack_messages::SlackEventMessageHandler, runtime_context::RuntimeContext};

// https://api.slack.com/apis/connections/events-api#handshake
#[derive(Deserialize, Debug)]
struct TopLevelContent {
    r#type: String,
}

// https://api.slack.com/apis/connections/events-api#handshake
#[derive(Deserialize, Debug)]
struct Handshake {
    challenge: String,
}

pub struct SlackEventHandler {
    message_handler: Arc<SlackEventMessageHandler>,
}

impl SlackEventHandler {
    pub fn new(runtime_context: &Arc<RuntimeContext>) -> Arc<Self> {
        let message_handler = SlackEventMessageHandler::new(runtime_context);
        let handler = Self {
            message_handler,
        };
        Arc::new(handler)
    }

    pub async fn handle_verified_events(&self, event: Request) -> Result<Response<Body>> {
        let Body::Text(body) = event.body() else {
            bail!("no body");
        };
        let content: TopLevelContent = serde_json::from_str(body)?;
        match content.r#type.as_str() {
            "url_verification" => self.url_verification(event),
            "event_callback" => self.event_callback(event, content.r#type).await,
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

    fn url_verification(&self, event: Request) -> Result<Response<Body>> {
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

    // https://api.slack.com/apis/connections/events-api#responding
    async fn event_callback(&self, event: Request, event_type: String) -> Result<Response<Body>> {
        let Body::Text(body) = event.body() else {
            bail!("no body");
        };
        self.message_handler.process_event_callback(event_type, body.clone()).await?;
        // respond to events with a HTTP 200 OK as soon as we can
        let response = Response::builder()
            .status(200)
            .header("content-type", "text/plain")
            .body("ok".into())
            .map_err(Box::new)?;
        Ok(response)
    }
}
