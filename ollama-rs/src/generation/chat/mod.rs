use serde::{Deserialize, Serialize};

use super::{images::Image, tools::ToolCall};
use crate::{error::OllamaError, history::ChatHistory, Ollama};
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
pub type ChatMessageResponseStream =
    std::pin::Pin<Box<dyn tokio_stream::Stream<Item = Result<ChatMessageResponse, ()>> + Send>>;

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
            let mut buffer = String::new();

            let mut stream = res.bytes_stream();
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        // Convert bytes to string and append to buffer
                        if let Ok(chunk_str) = String::from_utf8(chunk.to_vec()) {
                            buffer.push_str(&chunk_str);

                            // Process complete lines in the buffer
                            let mut lines_to_process = Vec::new();
                            let mut start_pos = 0;

                            // Find all complete lines in the buffer and collect them
                            while let Some(pos) = buffer[start_pos..].find('\n') {
                                let actual_pos = start_pos + pos;
                                let line = buffer[start_pos..actual_pos].trim().to_string();
                                if !line.is_empty() {
                                    lines_to_process.push(line);
                                }
                                start_pos = actual_pos + 1;
                            }

                            // If we processed any lines, truncate the buffer
                            if start_pos > 0 {
                                buffer = buffer[start_pos..].to_string();
                            }

                            // Process all collected lines
                            for line in lines_to_process {
                                // Parse the JSON line
                                match serde_json::from_str::<ChatMessageResponse>(&line) {
                                    Ok(response) => yield Ok(response),
                                    Err(e) => {
                                        eprintln!("Failed to deserialize response: {e}");
                                        // Continue processing other lines even if one fails
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read response: {e}");
                        yield Err(());
                        break;
                    }
                }
            }

            // Process any remaining data in the buffer
            if !buffer.is_empty() {
                if let Ok(response) = serde_json::from_str::<ChatMessageResponse>(&buffer) {
                    yield Ok(response);
                }
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

            while let Some(item) = resp_stream.try_next().await.unwrap() {
                let msg_part = item.clone().message.content;

                if item.done {
                    history.lock().unwrap().push(ChatMessage::assistant(result.clone()));
                } else {
                    result.push_str(&msg_part);
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
    pub done: bool,
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
}

impl ChatMessage {
    pub fn new(role: MessageRole, content: String) -> Self {
        Self {
            role,
            content,
            tool_calls: vec![],
            images: None,
            thinking: None,
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
