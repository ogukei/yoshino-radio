
use std::sync::Arc;

use anyhow::Result;
use anyhow::bail;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tracing::info;
use tokio::net::TcpStream;

use cores::ipc::{ChannelMessage, IPC_EXTENSION_ENDPOINT, IPC_ACCEPT_ACK_TOKEN};

pub struct ChannelClient {}

impl ChannelClient {
    pub fn new() -> Arc<Self> {
        let client = Self {
        };
        Arc::new(client)
    }

    pub async fn invoke(self: &Arc<Self>, message: ChannelMessage) -> Result<()> {
        info!("invoke in progress");
        let mut stream = TcpStream::connect(IPC_EXTENSION_ENDPOINT).await?;
        // ensure the extension is ready to receive messages
        let mut buffer = IPC_ACCEPT_ACK_TOKEN.clone();
        stream.read_exact(&mut buffer).await?;
        if buffer.to_vec() != IPC_ACCEPT_ACK_TOKEN.to_vec() {
            bail!("ack failure");
        }
        // write
        let content = serde_json::to_string(&message)?;
        stream.write_all(content.as_bytes()).await?;
        stream.shutdown().await?;
        info!("invoke complete");
        Ok(())
    }
} 
