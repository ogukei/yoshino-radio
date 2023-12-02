use std::sync::Arc;

use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response, http::Method};
use runtime_app::RuntimeApp;

mod runtime_app;
mod runtime_context;
mod channel_client;
mod slack_requests;
mod slack_events;
mod slack_messages;
mod slack_verification;

use runtime_context::RuntimeContext;
use slack_requests::SlackRequestHandler;

// https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
async fn function_handler(event: Request, context: &Arc<RuntimeContext>) -> Result<Response<Body>, Error> {
    match (event.method(), event.raw_http_path()) {
        (&Method::POST, "/slack/events") => {
            let request_handler = SlackRequestHandler::new(context);
            request_handler.handle_slack_request(event).await
        },
        (&Method::GET, "/") => {
            handle_get_root(event).await
        },
        _ => {
            handle_not_found(event).await
        }
    }
}

async fn handle_get_root(_event: Request) -> Result<Response<Body>, Error> {
    let response = Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body("Hello world".into())
        .map_err(Box::new)?;
    Ok(response)
}

async fn handle_not_found(_event: Request) -> Result<Response<Body>, Error> {
    let response = Response::builder()
        .status(404)
        .header("content-type", "text/plain")
        .body("not found".into())
        .map_err(Box::new)?;
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();
    let runtime_context = RuntimeContext::new();
    let runtime_app = RuntimeApp::new(&runtime_context);
    runtime_app.launch().await?;
    let func = |event| async {
        function_handler(event, &runtime_context).await
    };
    run(service_fn(func)).await
}
