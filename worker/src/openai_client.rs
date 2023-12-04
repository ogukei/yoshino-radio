
use std::fmt;
use std::{sync::Arc, env};
use anyhow::{Result, bail};
use reqwest::Response;
use reqwest::{self, Client};
use serde::{Deserialize, Serialize};
use tracing::info;
use futures_util::StreamExt;
use futures_util::Stream;

#[derive(Serialize)]
struct CompletionsRequestBody {
    model: String,
    messages: Vec<CompletionsRequestMessage>,
    stream: bool,
    max_tokens: u64,
}

#[derive(Serialize, Debug)]
pub struct CompletionsRequestMessage {
    pub role: String,
    pub content: Vec<CompletionsRequestMessageContent>,
}

#[derive(Serialize, Debug)]
pub struct CompletionsRequestMessageContent {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<CompletionsRequestMessageImageURL>,
}

#[derive(Serialize)]
pub struct CompletionsRequestMessageImageURL {
    pub url: String,
    pub detail: String,
}

impl fmt::Debug for CompletionsRequestMessageImageURL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.url.len() < 1024 {
            f.debug_struct("CompletionsRequestMessageImageURL")
                .field("url", &self.url)
                .finish()
        } else {
            f.debug_struct("CompletionsRequestMessageImageURL")
                .field("url", &format!("<string length {}>", self.url.len()))
                .finish()
        }
    }
}

pub struct OpenAIClient {
    client: Client,
}

// The chat completion chunk object
// https://platform.openai.com/docs/api-reference/chat/streaming
#[derive(Deserialize, Debug)]
pub struct CompletionsMessageChunk {
    pub id: String,
    pub choices: Vec<CompletionsMessageChunkChoise>,
}

#[derive(Deserialize, Debug)]
pub struct CompletionsMessageChunkChoise {
    pub delta: CompletionsMessageChunkDelta,
    pub finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CompletionsMessageChunkDelta {
    pub content: Option<String>,
    pub role: Option<String>,
}

// @see https://api.slack.com/rtm#sending_messages

impl OpenAIClient {
    pub fn new() -> Result<Arc<Self>> {
        let client = reqwest::Client::new();
        let this = Self {
            client,
        };
        let this = Arc::new(this);
        Ok(this)
    }

    // https://platform.openai.com/docs/api-reference/chat/create
    pub async fn completions(&self, messages: Vec<CompletionsRequestMessage>) -> Result<impl Stream<Item = Result<Vec<CompletionsMessageChunk>, anyhow::Error>>> {
        let response = self.completions_response(messages).await?;
        // check status
        if !response.status().is_success() {
            let text = response.text().await?;
            bail!("completions response failure. {}", text);
        }
        // https://docs.rs/reqwest/latest/reqwest/struct.Response.html#method.bytes_stream
        let stream = response.bytes_stream()
            .map(|bytes| -> Result<Vec<CompletionsMessageChunk>, anyhow::Error> {
                let bytes = bytes?;
                let bytes_slice: &[u8] = &bytes;
                let string = String::from_utf8(bytes_slice.to_vec())?;
                Self::process_completion_chunks(string)
            });
        Ok(stream)
    }

    fn process_completion_chunks(string: String) -> Result<Vec<CompletionsMessageChunk>, anyhow::Error> {
        // The data field for the message. When the EventSource receives multiple consecutive lines that begin with data:
        // https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#event_stream_format
        let lines: Vec<Option<CompletionsMessageChunk>> = string.lines()
            .map(|line| -> Option<CompletionsMessageChunk> {
                if let Some(data) = line.strip_prefix("data: ") {
                    // the stream terminated by a data: [DONE] message.
                    if !data.is_empty() && data != "[DONE]" {
                        info!("chunk {:?}", data);
                        let chunk: Option<CompletionsMessageChunk> = serde_json::from_str(data).ok();
                        chunk
                    } else {
                        None
                    }
                } else {
                    if !line.is_empty() {
                        info!("ignoring line {:?}", line);
                    }
                    None
                }
            })
            .collect();
        let lines: Vec<CompletionsMessageChunk> = lines.into_iter()
            .filter_map(|v| v)
            .collect();
        Ok(lines)
    }

    // https://platform.openai.com/docs/guides/vision
    async fn completions_response(&self, messages: Vec<CompletionsRequestMessage>) -> Result<Response> {
        let api_key = env::var("OPENAI_API_KEY")?;
        let request_body = CompletionsRequestBody {
            model: "gpt-4-vision-preview".into(),
            messages,
            max_tokens: 2048,
            stream: true,
        };
        let response = self.client.post("https://api.openai.com/v1/chat/completions")
            .header("Content-type", "application/json; charset=utf-8")
            .header("Authorization", ["Bearer", &api_key].join(" "))
            .json(&request_body)
            .send()
            .await?;
        info!("completions response {:?}", response);
        Ok(response)
    }
}
