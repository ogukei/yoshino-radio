use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response, http::Method};

mod slack_verification;
mod slack_events;

use crate::slack_verification::verify_slack_request;
use crate::slack_events::handle_verified_slack_events;

// https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    match (event.method(), event.raw_http_path()) {
        (&Method::POST, "/slack/events") => {
            handle_slack_events(event).await
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

async fn handle_slack_events(event: Request) -> Result<Response<Body>, Error> {
    let verification_result = verify_slack_request(&event);
    match verification_result {
        Ok(()) => {
            let result = handle_verified_slack_events(event).await;
            match result {
                Ok(response) => Ok(response),
                Err(error) => {
                    tracing::info!("/slack/events error {:?}", error);
                    let response = Response::builder()
                        .status(500)
                        .header("content-type", "text/plain")
                        .body("internal server error".into())
                        .map_err(Box::new)?;
                    Ok(response)
                }
            }
        },
        Err(error) => {
            tracing::info!("/slack/events verification failed {:?}", error);
            let response = Response::builder()
                .status(403)
                .header("content-type", "text/plain")
                .body("forbidden".into())
                .map_err(Box::new)?;
            Ok(response)
        }
    }
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
