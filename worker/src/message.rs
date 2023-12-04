
use std::{sync::Arc, time::Duration};

use anyhow::{Result, Context};
use cores::ipc::InvokeMessage;
use serde::Deserialize;
use tracing::info;
use futures_util::StreamExt;
use crate::{
    slack_client::SlackClient, 
    openai_client::{OpenAIClient, CompletionsRequestMessage, CompletionsMessageChunk, CompletionsRequestMessageContent, CompletionsRequestMessageImageURL},
    buffer_stream::periodic_buffered_window, images::ImageProcess};

use serde_json;
use crate::completions::Completions;

#[derive(Deserialize)]
struct AnyEventCallbackBody {
    event: AnyEvent,
}

#[derive(Deserialize)]
struct AnyEvent {
    r#type: String,
    subtype: Option<String>,
    bot_id: Option<String>,
}

#[derive(Deserialize)]
struct MessageEventCallbackBody {
    event: MessageEvent,
}

#[derive(Deserialize)]
struct MessageEvent {
    text: String,
    channel: String,
    ts: String,
    thread_ts: Option<String>,
    files: Option<Vec<MessageFile>>,
}

// https://api.slack.com/events/message/file_share
// https://api.slack.com/types/file
#[derive(Deserialize)]
struct MessageFile {
    id: String,
    mimetype: String,
    url_private_download: Option<String>,
    size: u64,
}

pub struct MessageHandle {
    slack_client: Arc<SlackClient>,
    openai_client: Arc<OpenAIClient>,
}

impl MessageHandle {
    pub fn new() -> Result<Arc<Self>> {
        let slack_client = SlackClient::new()?;
        let openai_client = OpenAIClient::new()?;
        let this = Self {
            slack_client,
            openai_client,
        };
        let this = Arc::new(this);
        Ok(this)
    }

    // https://api.slack.com/events/message.im
    pub async fn handle_message(&self, message: InvokeMessage) -> Result<()> {
        info!("worker received {}, {}", message.event_type, message.body);
        match message.event_type.as_str() {
            "event_callback" => self.handle_slack_event_callback(&message.body).await,
            _ => Ok(())
        }
    }

    async fn handle_slack_event_callback(&self, body: &str) -> Result<()> {
        let any_event: AnyEventCallbackBody = serde_json::from_str(&body)?;
        let r#type = any_event.event.r#type.as_str();
        // ignore message updates
        let subtype = any_event.event.subtype
            .as_ref()
            .map(|v| v.as_str());
        // ignore bot's messages
        let bot_id = any_event.event.bot_id;
        if bot_id.is_some() {
            return Ok(())
        }
        match (r#type, subtype) {
            ("message", None) | ("message", Some("file_share")) => self.handle_slack_message(body).await,
            _ => Ok(()),
        }
    }

    async fn handle_slack_message(&self, body: &str) -> Result<()> {
        let body: MessageEventCallbackBody = serde_json::from_str(body)?;
        let message_event = body.event;
        let text = &message_event.text;
        let channel = &message_event.channel;
        let post_result = self.slack_client.post(
            channel,
            Some(&message_event.ts),
             format!("Hi! `[Processing {}...]`", text)).await?;
        let thread_ts = post_result.thread_ts
            .as_ref()
            .context("missing thread_ts")?;
        // get image
        let file_image_url = self.file_image_url(&message_event).await?;
        // get replies
        let replies = self.slack_client.replies(channel, thread_ts).await?;
        // construct completions request
        let messages: Vec<CompletionsRequestMessage> = replies.messages.into_iter()
            .filter_map(|message| {
                let message = match (message.r#type.as_str(), &message.bot_id) {
                    ("message", None) if &message.ts == &message_event.ts => {
                        // https://platform.openai.com/docs/guides/vision/uploading-base-64-encoded-images
                        if let Some(ref file_image_url) = file_image_url {
                            let image_url = CompletionsRequestMessageImageURL {
                                url: file_image_url.into(),
                                detail: "low".into(),
                            };
                            CompletionsRequestMessage {
                                role: "user".into(),
                                content: vec![
                                    CompletionsRequestMessageContent {
                                        r#type: "text".into(),
                                        text: Some(message.text),
                                        image_url: None,
                                    },
                                    CompletionsRequestMessageContent {
                                        r#type: "image_url".into(),
                                        text: None,
                                        image_url: Some(image_url),
                                    },
                                ]
                            }
                        } else {
                            CompletionsRequestMessage {
                                role: "user".into(),
                                content: vec![
                                    CompletionsRequestMessageContent {
                                        r#type: "text".into(),
                                        text: Some(message.text),
                                        image_url: None,
                                    }
                                ]
                            }
                        }
                    },
                    ("message", None) if &message.ts != &message_event.ts => {
                        CompletionsRequestMessage {
                            role: "user".into(),
                            content: vec![
                                CompletionsRequestMessageContent {
                                    r#type: "text".into(),
                                    text: Some(message.text),
                                    image_url: None,
                                }
                            ]
                        }
                    }
                    ("message", Some(_)) => CompletionsRequestMessage {
                        role: "assistant".into(),
                        content: vec![
                            CompletionsRequestMessageContent {
                                r#type: "text".into(),
                                text: Some(message.text),
                                image_url: None,
                            }
                        ]
                    },
                    _ => return None,
                };
                Some(message)
            })
            .collect();
        let messages = {
            let mut v = Self::system_messages();
            v.extend(messages);
            v
        };
        info!("completions request messages {:?}", messages);
        // run completions
        let completions = Completions::new(&self.openai_client)?;
        let mut content_stream = completions.periodic_contents(messages).await?;
        while let Some(content) = content_stream.next().await {
            self.slack_client.update(channel, &post_result.ts, content.clone()).await?;
        }
        info!("completions complete!");
        Ok(())
    }

    async fn file_image_url(&self, message_event: &MessageEvent) -> Result<Option<String>> {
        let Some(files) = &message_event.files else { return Ok(None) };
        let Some(file) = files.first() else { return Ok(None) };
        let Some(ref url_private_download) = file.url_private_download else { return Ok(None) };
        let mimetype = &file.mimetype;
        // check mimetype
        match mimetype.as_str() {
            "image/jpeg" | "image/png" => (),
            _ => return Ok(None), 
        };
        info!("downloading file {}...", file.id);
        let data = self.slack_client.download_image(url_private_download).await?;
        let data_base64 = ImageProcess::new()?.base64(data)?;
        let data_url = format!("data:{mimetype};base64,{data_base64}");
        info!("download file complete: size {}...", data_url.len());
        Ok(Some(data_url))
    }

    // https://platform.openai.com/tokenizer
    fn system_messages() -> Vec<CompletionsRequestMessage> {
        // 704 tokens
        // special thanks: 
        // https://seesaawiki.jp/yoshino_yorita/d/%b0%cd%c5%c4%cb%a7%c7%b5%a4%c8%a4%cf
        let text = r#"
あなたは依田芳乃です。依田芳乃は皆の力になってくれるアイドルです。
皆からは「よしのん」と呼ばれています。一人称は「わたくし」です。二人称は「そなた」です。

性格は人の悩みを聞いたり他人を受け入れる大らかさと自分の使命を果たそうとする真っすぐさがあり他人思いです。
趣味は悩み事解決・石ころ集め・失せ物探しの3つです。
好きな食べ物はお煎餅です。世俗的な文化や海外の文化には疎いですが克服しようとしています。

セリフの例を出すので参考にしてください。
「わたくし依田は芳乃と申しましてー」
「それが、わたくしの幸せでしてー」
「ここには魂や言ノ葉が詰まっておりー。ゆえに特別な美しさもありましてー」
「人の世に穢れは付き物でしてー。だからこそ、人は癒しを必要とするのですよー」
「宿命を背負おうとも、共にあればー。風を吹かせ、花を舞わせましょー。それがわたくしの在る意味でしてー」
「朗らかにー笑ってくださいー。それがわたくしの力になりますゆえー」
「さらなる高みを目指しながらも、みなのそばにー。そんなあいどるになるものでしょうー」
「お悩みがあるのでしょうかー？ならばこそ任せるのですー」
「待ち人は北にと、書いてありましてー」
「荒ぶる心やいらだちを、お鎮めしたくー」
「包まれるとぬくぬくとしましてー」
「悩みを抱える者がいるのならば、わたくしもともに悩みましょうー」
「横文字の言葉は、いまだ慣れませぬー。あるふぁべっとなど、特に…」
「ぱしゃぱしゃー。ふふー、冷たい水が心地良いですねー。それ、ぱしゃー」
"#;
        vec![
            CompletionsRequestMessage {
                role: "system".into(),
                content: vec![
                    CompletionsRequestMessageContent {
                        r#type: "text".into(),
                        text: Some(text.into()),
                        image_url: None,
                    }
                ]
            }
        ]
    }
}
