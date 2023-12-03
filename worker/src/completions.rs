

use std::sync::Arc;
use std::time::Duration;

use async_stream::stream;

use crate::buffer_stream::periodic_buffered_window;
use crate::openai_client::CompletionsRequestMessage;
use crate::openai_client::OpenAIClient;
use crate::openai_client::CompletionsMessageChunk;

use anyhow::Result;
use futures_util::StreamExt;
use futures_util::Stream;
use futures_util::future;

pub struct Completions {
    client: Arc<OpenAIClient>,
}

impl Completions {
    pub fn new(client: &Arc<OpenAIClient>) -> Result<Arc<Self>> {
        let client = Arc::clone(client);
        let this = Self { client };
        let this = Arc::new(this);
        Ok(this)
    }

    // convenience wrapper for completions()
    pub async fn periodic_contents(&self, messages: Vec<CompletionsRequestMessage>) -> Result<impl Stream<Item = String>> {
        let contents = self.concatenated_contents(messages).await?;
        let windows = periodic_buffered_window(Duration::from_secs(1), contents);
        let latest_content_stream = windows
            .filter_map(|v| {
                let v = v.last().map(String::clone);
                future::ready(v)
            })
            .boxed();
        Ok(latest_content_stream)
    }

    async fn concatenated_contents(&self, messages: Vec<CompletionsRequestMessage>) -> Result<impl Stream<Item = String>> {
        let completions_stream = self.client.completions(messages).await?;
        let content_stream = stream! {
            let mut concatenated_content = String::new();
            let mut completions_stream = completions_stream;
            while let Some(item) = completions_stream.next().await {
                let Ok(chunks) = item else { continue };
                let chunks: Vec<CompletionsMessageChunk> = chunks;
                for chunk in chunks {
                    for choise in chunk.choices {
                        if let Some(content) = choise.delta.content {
                            concatenated_content += &content;
                            if !concatenated_content.is_empty() {
                                yield concatenated_content.clone();
                            }
                        }
                    }
                }
            }
        };
        Ok(content_stream)
    }
}
