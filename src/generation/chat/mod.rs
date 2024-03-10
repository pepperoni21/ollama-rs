use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Ollama;

pub mod request;

use request::ChatMessageRequest;

use super::images::Image;

#[cfg(feature = "stream")]
/// A stream of `ChatMessageResponse` objects
pub type ChatMessageResponseStream =
    std::pin::Pin<Box<dyn tokio_stream::Stream<Item = Result<ChatMessageResponse, ()>> + Send>>;

impl Ollama {
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

        let uri = format!("{}/api/chat", self.uri());
        let serialized = serde_json::to_string(&request)
            .map_err(|e| e.to_string())
            .unwrap();
        let res = self
            .reqwest_client
            .post(uri)
            .body(serialized)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(res.text().await.unwrap_or_else(|e| e.to_string()).into());
        }

        let stream = Box::new(res.bytes_stream().map(|res| match res {
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

        let uri = format!("{}/api/chat", self.uri());
        let serialized = serde_json::to_string(&request).map_err(|e| e.to_string())?;
        let res = self
            .reqwest_client
            .post(uri)
            .body(serialized)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(res.text().await.unwrap_or_else(|e| e.to_string()).into());
        }

        let bytes = res.bytes().await.map_err(|e| e.to_string())?;
        let res =
            serde_json::from_slice::<ChatMessageResponse>(&bytes).map_err(|e| e.to_string())?;

        Ok(res)
    }
}

#[cfg(feature = "chat-history")]
impl Ollama {
    /// Chat message generation
    /// Returns a `ChatMessageResponse` object
    /// Manages the history of messages for the given `id`
    pub async fn send_chat_messages_with_history(
        &mut self,
        mut request: ChatMessageRequest,
        id: String,
    ) -> crate::error::Result<ChatMessageResponse> {
        let mut backup = MessagesHistory::default();

        let current_chat_messages = self
            .messages_history
            .as_mut()
            .unwrap_or(&mut backup)
            .messages_by_id
            .entry(id.clone())
            .or_default();

        current_chat_messages.push(request.messages[0].clone());

        request.messages = current_chat_messages.clone();

        let result = self.send_chat_messages(request).await;

        if let Ok(result) = result {
            self.messages_history
                .as_mut()
                .unwrap_or(&mut backup)
                .add_message(id, result.message.clone().unwrap());
            return Ok(result);
        }

        result
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatMessageResponse {
    /// The name of the model used for the completion.
    pub model: String,
    /// The creation time of the completion, in such format: `2023-08-04T08:52:19.385406455-07:00`.
    pub created_at: String,
    /// The generated chat message.
    pub message: Option<ChatMessage>,
    pub done: bool,
    #[serde(flatten)]
    /// The final data of the completion. This is only present if the completion is done.
    pub final_data: Option<ChatMessageFinalResponseData>,
}

#[derive(Debug, Clone, Deserialize)]
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
    pub images: Option<Vec<Image>>,
}

impl ChatMessage {
    pub fn new(role: MessageRole, content: String) -> Self {
        Self {
            role,
            content,
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

#[cfg(feature = "chat-history")]
#[derive(Debug, Clone, Default)]
pub struct MessagesHistory {
    pub(crate) messages_by_id: HashMap<String, Vec<ChatMessage>>,
    pub(crate) messages_number_limit: u16,
}

#[cfg(feature = "chat-history")]
impl MessagesHistory {
    pub fn new(messages_number_limit: u16) -> Self {
        Self {
            messages_by_id: HashMap::new(),
            messages_number_limit: messages_number_limit.max(2),
        }
    }

    pub fn add_message(&mut self, entry_id: String, message: ChatMessage) {
        let messages = self.messages_by_id.entry(entry_id).or_default();

        // Replacing the oldest message if the limit is reached
        // The oldest message is the first one, unless it's a system message
        if messages.len() >= self.messages_number_limit as usize {
            let index_to_remove = messages
                .first()
                .map(|m| if m.role == MessageRole::System { 1 } else { 0 })
                .unwrap_or(0);

            messages.remove(index_to_remove);
        }

        if message.role == MessageRole::System {
            messages.insert(0, message);
        } else {
            messages.push(message);
        }
    }

    pub fn get_messages(&self, entry_id: &str) -> Option<&Vec<ChatMessage>> {
        self.messages_by_id.get(entry_id)
    }

    pub fn clear_messages(&mut self, entry_id: &str) {
        self.messages_by_id.remove(entry_id);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
}
