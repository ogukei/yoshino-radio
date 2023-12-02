
use std::sync::Arc;
use crate::runtime_context::RuntimeContext;

use anyhow::Result;

pub struct RuntimeApp {
    runtime_context: Arc<RuntimeContext>,
}

impl RuntimeApp {
    pub fn new(runtime_context: &Arc<RuntimeContext>) -> Arc<Self> {
        let runtime_context = Arc::clone(runtime_context);
        let handler = Self {
            runtime_context,
        };
        Arc::new(handler)
    }

    pub async fn launch(self: &Arc<Self>) -> Result<()> {
        Ok(())
    }
}
