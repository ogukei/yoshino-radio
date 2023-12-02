
use std::sync::Arc;

use lambda_http::Error;
use lambda_http::{Body, Request, Response};

use crate::{slack_events::SlackEventHandler, runtime_context::RuntimeContext};
use crate::slack_verification::verify_slack_request;

pub struct SlackRequestHandler {
    event_handler: Arc<SlackEventHandler>,
}

impl SlackRequestHandler {
    pub fn new(runtime_context: &Arc<RuntimeContext>) -> Arc<Self> {
        let event_handler = SlackEventHandler::new(runtime_context);
        let handler = Self {
            event_handler,
        };
        Arc::new(handler)
    }

    pub async fn handle_slack_request(&self, event: Request) -> Result<Response<Body>, Error> {
        let verification_result = verify_slack_request(&event);
        match verification_result {
            Ok(()) => {
                let result = self.event_handler.handle_verified_events(event).await;
                match result {
                    Ok(response) => Ok(response),
                    Err(error) => {
                        tracing::info!("/slack/events error {:?}", error);
                        self.internal_server_error_response()
                    }
                }
            },
            Err(error) => {
                tracing::info!("/slack/events verification failed {:?}", error);
                self.forbidden_response()
            }
        }
    }

    fn internal_server_error_response(&self) -> Result<Response<Body>, Error> {
        let response = Response::builder()
            .status(500)
            .header("content-type", "text/plain")
            .body("internal server error".into())
            .map_err(Box::new)?;
        Ok(response)
    }

    fn forbidden_response(&self) -> Result<Response<Body>, Error> {
        let response = Response::builder()
            .status(403)
            .header("content-type", "text/plain")
            .body("forbidden".into())
            .map_err(Box::new)?;
        Ok(response)
    }
}
