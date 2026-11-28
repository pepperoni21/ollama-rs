use serde::{Deserialize, Serialize};

use super::{images::Image, tools::ToolCall};
#[cfg(feature = "stream")]
use crate::error::InternalOllamaError;
use crate::{
    error::OllamaError, generation::parameters::LogprobsData, history::ChatHistory, Ollama,
};
use request::ChatMessageRequest;

#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
use async_stream::stream;
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
use std::sync::{Arc, Mutex};
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
use tokio_stream::StreamExt;

pub mod request;

#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
/// A stream of `ChatMessageResponse` objects
pub type ChatMessageResponseStream = std::pin::Pin<
    Box<dyn tokio_stream::Stream<Item = crate::error::Result<ChatMessageResponse>> + Send>,
>;

/// Parse one NDJSON line from a streaming chat response.
///
/// A well-formed line deserializes into a [`ChatMessageResponse`]. Ollama can
/// also terminate a stream with an error line of the shape `{"error": "..."}`
/// (e.g. a mid-generation CUDA OOM); we surface that as
/// [`OllamaError::InternalError`] so the message is not lost. Anything else is a
/// genuine deserialization failure and is returned as [`OllamaError::JsonError`].
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
fn parse_chat_line(line: &[u8]) -> crate::error::Result<ChatMessageResponse> {
    match serde_json::from_slice::<ChatMessageResponse>(line) {
        Ok(response) => Ok(response),
        Err(orig) => match serde_json::from_slice::<InternalOllamaError>(line) {
            Ok(internal) => Err(OllamaError::InternalError(internal)),
            Err(_) => Err(OllamaError::JsonError(orig)),
        },
    }
}

impl Ollama {
    #[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
    #[cfg(feature = "stream")]
    /// Chat message generation with streaming.
    /// Returns a stream of `ChatMessageResponse` objects
    pub async fn send_chat_messages_stream(
        &self,
        request: ChatMessageRequest,
    ) -> crate::error::Result<ChatMessageResponseStream> {
        let mut request = request;
        request.stream = true;

        let url = format!("{}api/chat", self.url_str());
        let builder = self.reqwest_client.post(url);

        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());

        let res = builder.json(&request).send().await?;

        if !res.status().is_success() {
            return Err(OllamaError::Other(
                res.text().await.unwrap_or_else(|e| e.to_string()),
            ));
        }

        let s = stream! {
            let mut line = Vec::new();
            let mut stream = res.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let chunk = match chunk {
                    Ok(chunk) => chunk,
                    Err(err) => {
                        yield Err(err.into());
                        return;
                    }
                };
                // HTTP chunks can split a UTF-8 code point as well as a JSON record.
                for part in chunk.split_inclusive(|&byte| byte == b'\n') {
                    line.extend_from_slice(part);
                    if part.ends_with(b"\n") {
                        if !line.iter().all(u8::is_ascii_whitespace) {
                            match parse_chat_line(&line) {
                                Ok(response) => yield Ok(response),
                                Err(err) => {
                                    yield Err(err);
                                    return;
                                }
                            }
                        }
                        line.clear();
                    }
                }
            }
            if !line.iter().all(u8::is_ascii_whitespace) {
                yield parse_chat_line(&line);
            }
        };

        Ok(Box::pin(s))
    }

    /// Chat message generation.
    /// Returns a `ChatMessageResponse` object
    pub async fn send_chat_messages(
        &self,
        request: ChatMessageRequest,
    ) -> crate::error::Result<ChatMessageResponse> {
        let mut request = request;
        request.stream = false;

        let url = format!("{}api/chat", self.url_str());
        let builder = self.reqwest_client.post(url);

        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());

        let res = builder.json(&request).send().await?;

        if !res.status().is_success() {
            return Err(OllamaError::Other(
                res.text().await.unwrap_or_else(|e| e.to_string()),
            ));
        }

        let bytes = res.bytes().await?;
        let res = serde_json::from_slice::<ChatMessageResponse>(&bytes)?;

        Ok(res)
    }
}

impl Ollama {
    #[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
    #[cfg(feature = "stream")]
    pub async fn send_chat_messages_with_history_stream<C: ChatHistory + Send + 'static>(
        &self,
        history: Arc<Mutex<C>>,
        mut request: ChatMessageRequest,
    ) -> crate::error::Result<ChatMessageResponseStream> {
        use async_stream::stream;
        use tokio_stream::StreamExt;

        // The request is modified to include the current chat messages
        {
            let mut hist = history.lock().unwrap();
            for m in request.messages {
                hist.push(m);
            }
        }

        request.messages = history.lock().unwrap().messages().to_vec();
        request.stream = true;

        let mut resp_stream: ChatMessageResponseStream =
            self.send_chat_messages_stream(request.clone()).await?;

        let s = stream! {
            let mut result = String::new();
            let mut tool_calls = Vec::new();

            while let Some(item) = resp_stream.next().await {
                let mut item = match item {
                    Ok(item) => item,
                    Err(e) => {
                        yield Err(e);
                        return;
                    }
                };
                result.push_str(&item.message.content);

                if item.done {
                    item.message.content = std::mem::take(&mut result);
                    tool_calls.append(&mut item.message.tool_calls);
                    item.message.tool_calls = std::mem::take(&mut tool_calls);
                    history.lock().unwrap().push(item.message.clone());
                } else {
                    tool_calls.extend_from_slice(&item.message.tool_calls);
                }

                yield Ok(item);
            }
        };

        Ok(Box::pin(s))
    }

    /// Chat message generation
    /// Returns a `ChatMessageResponse` object
    pub async fn send_chat_messages_with_history<C: ChatHistory>(
        &self,
        history: &mut C,
        mut request: ChatMessageRequest,
    ) -> crate::error::Result<ChatMessageResponse> {
        // The request is modified to include the current chat messages
        for m in request.messages {
            history.push(m);
        }

        request.messages = history.messages().to_vec();

        let result = self.send_chat_messages(request.clone()).await;

        if let Ok(result) = result {
            history.push(result.message.clone());

            return Ok(result);
        }

        result
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageResponse {
    /// The name of the model used for the completion.
    pub model: String,
    /// The creation time of the completion, in such format: `2023-08-04T08:52:19.385406455-07:00`.
    pub created_at: String,
    /// The generated chat message.
    pub message: ChatMessage,
    /// The log probabilities (only if `logprobs` is set to `true`)
    pub logprobs: Option<Vec<LogprobsData>>,
    pub done: bool,
    /// Why generation stopped: `"stop"`, `"length"`, etc. Only present on the
    /// final (`done == true`) response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub done_reason: Option<String>,
    #[serde(flatten)]
    /// The final data of the completion. This is only present if the completion is done.
    pub final_data: Option<ChatMessageFinalResponseData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageFinalResponseData {
    /// Time spent generating the response
    pub total_duration: u64,
    /// Time spent in nanoseconds loading the model
    pub load_duration: u64,
    /// Number of tokens in the prompt
    pub prompt_eval_count: u64,
    /// Number of prompt tokens read from the cache
    pub prompt_eval_cached_count: Option<u64>,
    /// Time spent in nanoseconds evaluating the prompt
    pub prompt_eval_duration: u64,
    /// Number of tokens the response
    pub eval_count: u64,
    /// Time in nanoseconds spent generating the response
    pub eval_duration: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<Image>>,
    pub thinking: Option<String>,
    /// Name of the tool whose result this `tool` message carries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    /// ID of the tool call this `tool` message answers, when the model returned one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    pub fn new(role: MessageRole, content: String) -> Self {
        Self {
            role,
            content,
            tool_calls: vec![],
            images: None,
            thinking: None,
            tool_name: None,
            tool_call_id: None,
        }
    }

    pub fn user(content: String) -> Self {
        Self::new(MessageRole::User, content)
    }

    pub fn assistant(content: String) -> Self {
        Self::new(MessageRole::Assistant, content)
    }

    pub fn system(content: String) -> Self {
        Self::new(MessageRole::System, content)
    }

    pub fn tool(content: String) -> Self {
        Self::new(MessageRole::Tool, content)
    }

    /// Sets the name of the tool whose result this message carries.
    pub fn with_tool_name(mut self, tool_name: impl Into<String>) -> Self {
        self.tool_name = Some(tool_name.into());
        self
    }

    /// Sets the ID of the tool call this message answers.
    pub fn with_tool_call_id(mut self, tool_call_id: impl Into<String>) -> Self {
        self.tool_call_id = Some(tool_call_id.into());
        self
    }

    pub fn with_images(mut self, images: Vec<Image>) -> Self {
        self.images = Some(images);
        self
    }

    pub fn add_image(mut self, image: Image) -> Self {
        if let Some(images) = self.images.as_mut() {
            images.push(image);
        } else {
            self.images = Some(vec![image]);
        }
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
    #[serde(rename = "tool")]
    Tool,
}
