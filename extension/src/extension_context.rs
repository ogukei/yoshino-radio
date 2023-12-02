
use std::sync::Arc;

use tokio_util::task::TaskTracker;

pub struct ExtensionContext {
    task_tracker: TaskTracker,
}

impl ExtensionContext {
    pub fn new() -> Arc<Self> {
        let context = Self { task_tracker: TaskTracker::new() };
        Arc::new(context)
    }

    pub fn task_tracker(&self) -> &TaskTracker {
        &self.task_tracker
    }
}
