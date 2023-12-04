
use std::sync::Arc;

use crate::openai_client::OpenAIClient;
use anyhow::Result;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as Base64;

pub struct ImageProcess {
}

impl ImageProcess {
    pub fn new() -> Result<Arc<Self>> {
        let this = Self { };
        let this = Arc::new(this);
        Ok(this)
    }

    pub fn base64(&self, data: Vec<u8>) -> Result<String> {
        let encoded = Base64.encode(data);
        Ok(encoded)
    }
}
