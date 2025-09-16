use serde::{Deserialize, Serialize};

use super::{images::Image, tools::ToolCall};
use crate::{error::OllamaError, history::ChatHistory, Ollama};
use request::ChatMessageRequest;

pub mod request;

#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
/// A stream of `ChatMessageResponse` objects
pub type ChatMessageResponseStream =
    futures_util::stream::BoxStream<'static, Result<ChatMessageResponse, OllamaError>>;

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
        crate::stream::map_response(res).await
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
        history: std::sync::Arc<std::sync::Mutex<C>>,
        mut request: ChatMessageRequest,
    ) -> crate::error::Result<ChatMessageResponseStream> {
        // The request is modified to include the current chat messages

        use futures_util::StreamExt;
        {
            let mut hist = history.lock().unwrap();
            for m in request.messages {
                hist.push(m);
            }
        }

        request.messages = history.lock().unwrap().messages().to_vec();
        request.stream = true;
        let mut result = String::new();
        Ok(Box::pin(
            self.send_chat_messages_stream(request.clone())
                .await?
                .then(move |x| {
                    std::future::ready(x.map(|x| {
                        if x.done {
                            history
                                .lock()
                                .unwrap()
                                .push(ChatMessage::assistant(result.clone()));
                        } else {
                            let msg_part = &x.message.content;
                            result.push_str(&msg_part);
                        }
                        x
                    }))
                }),
        ))
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
