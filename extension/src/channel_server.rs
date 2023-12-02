
use std::sync::Arc;

use cores::ipc::{ChannelMessage, IPC_EXTENSION_ENDPOINT, IPC_ACCEPT_ACK_TOKEN};
use tokio::net::{TcpStream, TcpListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use anyhow::Result;
use tracing::info;

use crate::extension_context::ExtensionContext;
use crate::message::MessageHandle;

pub struct ChannelServer {
    context: Arc<ExtensionContext>,
    message_handle: Arc<MessageHandle>,
}

impl ChannelServer {
    pub fn new(context: &Arc<ExtensionContext>, message_handle: &Arc<MessageHandle>) -> Arc<Self> {
        let context = Arc::clone(context);
        let message_handle = Arc::clone(message_handle);
        let server = Self {
            context,
            message_handle,
        };
        Arc::new(server)
    }

    pub async fn bind(self: &Arc<Self>) -> Result<()> {
        info!("extension binding {}", IPC_EXTENSION_ENDPOINT);
        let listener = TcpListener::bind(IPC_EXTENSION_ENDPOINT).await?;
        info!("extension binding successful");
        let server = Arc::clone(self);
        let task = || async move {
            loop {
                let result = server.accept(&listener).await;
                match result {
                    Ok(()) => (),
                    Err(e) => {
                        info!("extension accept error {:?}", e);
                        break;
                    }
                }
            }
        };
        tokio::spawn(task());
        Ok(())
    }

    async fn accept(self: &Arc<Self>, listener: &TcpListener) -> Result<()> {
        let (stream, _) = listener.accept().await?;
        let server = Arc::clone(self);
        // our extension waits until the task is completed regardless of whether the runtime is shutdown
        // this task is required to finish
        let task = || async move {
            let result = server.receive(stream).await;
            match result {
                Ok(()) => (),
                Err(e) => {
                    info!("extension. receive error {:?}", e);
                }
            }
        };
        self.context.task_tracker().spawn(task());
        Ok(())
    }

    async fn receive(&self, mut stream: TcpStream) -> Result<()> {
        // notify the runtime that extension is ready to receive message
        self.ack_accept(&mut stream).await?;
        // receive message until EOF
        let mut content = String::new();
        let _ = stream.read_to_string(&mut content).await?;
        // handle
        let message: ChannelMessage = serde_json::from_str(&content)?;
        self.message_handle.handle_message(message).await?;
        Ok(())
    }

    async fn ack_accept(&self, stream: &mut TcpStream) -> Result<()> {
        stream.write_all(IPC_ACCEPT_ACK_TOKEN).await?;
        stream.flush().await?;
        Ok(())
    }
} 
