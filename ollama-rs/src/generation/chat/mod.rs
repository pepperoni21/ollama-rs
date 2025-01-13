use serde::{Deserialize, Serialize};

use crate::{error::OllamaError, history::ChatHistory, Ollama};
pub mod request;
use super::{images::Image, tools::ToolCall};
use request::ChatMessageRequest;

#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
use async_stream::stream;
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
use std::sync::{Arc, Mutex};

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
        use tokio_stream::StreamExt;

        let mut request = request;
        request.stream = true;

        let url = format!("{}api/chat", self.url_str());
        let serialized = serde_json::to_string(&request)
            .map_err(|e| e.to_string())
            .unwrap();
        let mut builder = self.reqwest_client.post(url);

        if let Some(timeout) = request.timeout {
            builder = builder.timeout(timeout);
        }

        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());

        let res = builder.body(serialized).send().await?;

        if !res.status().is_success() {
            return Err(OllamaError::Other(
                res.text().await.unwrap_or_else(|e| e.to_string()),
            ));
        }

        let stream = Box::new(res.bytes_stream().map(move |res| {
            if let Some(abort_signal) = request.abort_signal.as_ref() {
                if abort_signal.aborted() {
                    return Err(());
                }
            }
            match res {
                Ok(bytes) => {
                    let res = serde_json::from_slice::<ChatMessageResponse>(&bytes);
                    match res {
                        Ok(res) => Ok(res),
                        Err(e) => {
                            eprintln!("Failed to deserialize response: {}", e);
                            Err(())
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read response: {}", e);
                    Err(())
                }
            }
        }));

        Ok(std::pin::Pin::from(stream))
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
        let serialized = serde_json::to_string(&request)?;
        let mut builder = self.reqwest_client.post(url);

        if let Some(timeout) = request.timeout {
            builder = builder.timeout(timeout);
        }

        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());

        let res = builder.body(serialized).send().await?;

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
        &mut self,
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
    /// Number of tokens in the prompt
    pub prompt_eval_count: u16,
    /// Time spent in nanoseconds evaluating the prompt
    pub prompt_eval_duration: u64,
    /// Number of tokens the response
    pub eval_count: u16,
    /// Time in nanoseconds spent generating the response
    pub eval_duration: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
    pub images: Option<Vec<Image>>,
}

impl ChatMessage {
    pub fn new(role: MessageRole, content: String) -> Self {
        Self {
            role,
            content,
            tool_calls: vec![],
            images: None,
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
