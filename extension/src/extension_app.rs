
use std::sync::Arc;
use crate::{extension_context::ExtensionContext, channel_server::ChannelServer, message::MessageHandle};

use anyhow::Result;

pub struct ExtensionApp {
    extension_context: Arc<ExtensionContext>,
    message_handle: Arc<MessageHandle>,
}

impl ExtensionApp {
    pub fn new(extension_context: &Arc<ExtensionContext>) -> Arc<Self> {
        let extension_context = Arc::clone(extension_context);
        let message_handle = MessageHandle::new(&extension_context);
        let handler = Self {
            extension_context,
            message_handle,
        };
        Arc::new(handler)
    }

    pub async fn launch(self: &Arc<Self>) -> Result<()> {
        let server = ChannelServer::new(&self.extension_context, &self.message_handle);
        server.bind().await?;
        Ok(())
    }
}
