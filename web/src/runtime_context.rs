
use std::sync::Arc;
use tokio_util::task::TaskTracker;

use crate::channel_client::ChannelClient;

pub struct RuntimeContext {
    task_tracker: TaskTracker,
    channel_client: Arc<ChannelClient>,
}

impl RuntimeContext {
    pub fn new() -> Arc<Self> {
        let channel_client = ChannelClient::new();
        let context = Self {
            task_tracker: TaskTracker::new(),
            channel_client,
        };
        Arc::new(context)
    }

    pub fn task_tracker(&self) -> &TaskTracker {
        &self.task_tracker
    }

    pub fn channel_client(&self) -> &Arc<ChannelClient> {
        &self.channel_client
    }
}
