use cores::ipc::InvokeMessage;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::message::MessageHandle;

mod message;
mod slack_client;
mod openai_client;

#[derive(Serialize)]
struct Response {
    req_id: String,
}

async fn function_handler(event: LambdaEvent<InvokeMessage>) -> Result<Response, Error> {
    let message = event.payload;
    let handle = MessageHandle::new();
    info!("got body {}", message.body);
    handle.handle_message(message).await?;
    let resp = Response {
        req_id: event.context.request_id,
    };
    Ok(resp)
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

    run(service_fn(function_handler)).await
}
