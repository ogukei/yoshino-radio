
use std::sync::Arc;

use extension_app::ExtensionApp;
use lambda_extension::*;
use tracing::info;

mod channel_server;
mod extension_app;
mod extension_context;
mod message;
mod slack_client;

use extension_context::ExtensionContext;

async fn events_extension(event: LambdaEvent, extension_context: &Arc<ExtensionContext>) -> Result<(), Error> {
    match event.next {
        NextEvent::Shutdown(e) => {
            info!(event_type = "shutdown_begin", event = ?e, "shutdown in progress...");
            let task_tracker = extension_context.task_tracker();
            task_tracker.close();
            task_tracker.wait().await;
            info!(event_type = "shutdown_end", event = ?e, "shutdown complete");
        }
        NextEvent::Invoke(e) => {
            info!(event_type = "invoke", event = ?e, "invoking function");
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();
    let extension_context = ExtensionContext::new();
    let extension_app = ExtensionApp::new(&extension_context);
    extension_app.launch().await?;
    let func = |event| async {
        events_extension(event, &extension_context).await
    };
    Extension::new()
        .with_events_processor(service_fn(func))
        .run()
        .await
}
