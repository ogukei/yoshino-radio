
use std::sync::Arc;

use anyhow::Result;
use anyhow::bail;
use aws_config::BehaviorVersion;
use aws_sdk_lambda::primitives::Blob;
use aws_sdk_lambda::types::InvocationType;
use cores::ipc::InvokeMessage;
use tracing::info;

use aws_sdk_lambda::Client;

pub struct ChannelClient {}

impl ChannelClient {
    pub fn new() -> Arc<Self> {
        let client = Self {
        };
        Arc::new(client)
    }

    pub async fn invoke(self: &Arc<Self>, message: InvokeMessage) -> Result<()> {
        let config = aws_config::load_defaults(BehaviorVersion::v2023_11_09()).await;
        info!("invoke in progress");
        let payload = serde_json::to_string(&message)?;
        let client = Client::new(&config);
        client.invoke()
            .function_name("yoshino-radio-worker")
            .payload(Blob::new(payload))
            .invocation_type(InvocationType::Event)
            .send()
            .await?;
        info!("invoke complete");
        Ok(())
    }
}
