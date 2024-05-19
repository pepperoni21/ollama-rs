#[cfg(all(feature = "chat-history", feature = "stream"))]
use async_stream::stream;
use serde::{Deserialize, Serialize};

use crate::Ollama;
pub mod request;
use super::images::Image;
use request::ChatMessageRequest;

#[cfg(feature = "chat-history")]
use crate::history::MessagesHistory;
#[cfg(all(feature = "chat-history", feature = "stream"))]
use crate::history_async::MessagesHistoryAsync;

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

        let url = format!("{}api/chat", self.url_str());
        let serialized = serde_json::to_string(&request)
            .map_err(|e| e.to_string())
            .unwrap();
        let res = self
            .reqwest_client
            .post(url)
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

        let url = format!("{}api/chat", self.url_str());
        let serialized = serde_json::to_string(&request).map_err(|e| e.to_string())?;
        let res = self
            .reqwest_client
            .post(url)
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
    pub async fn send_chat_messages_with_history<S: Into<String> + Clone>(
        &mut self,
        mut request: ChatMessageRequest,
        history_id: S,
    ) -> crate::error::Result<ChatMessageResponse> {
        // The request is modified to include the current chat messages
        request.messages = self.get_prefill_messages(history_id.clone(), request.messages);

        let result = self.send_chat_messages(request).await;

        match result {
            Ok(result) => {
                // Store AI's response in the history
                self.store_chat_message_by_id(history_id, result.message.clone().unwrap());

                return Ok(result);
            }
            Err(_) => {
                self.remove_history_last_message(history_id);
            }
        }

        result
    }

    /// Helper function to store chat messages by id
    fn store_chat_message_by_id<S: Into<String>>(&mut self, id: S, message: ChatMessage) {
        if let Some(messages_history) = self.messages_history.as_mut() {
            messages_history.add_message(id, message);
        }
    }

    /// Let get existing history with a new message in it
    /// Without impact for existing history
    /// Used to prepare history for request
    fn get_prefill_messages<S: Into<String>>(
        &mut self,
        history_id: S,
        request_messages: Vec<ChatMessage>,
    ) -> Vec<ChatMessage> {
        let mut backup = MessagesHistory::default();

        // Clone the current chat messages to avoid borrowing issues
        // And not to add message to the history if the request fails
        let current_chat_messages = self
            .messages_history
            .as_mut()
            .unwrap_or(&mut backup)
            .messages_by_id
            .entry(history_id.into())
            .or_default();

        if let Some(message) = request_messages.first() {
            current_chat_messages.push(message.clone());
        }

        current_chat_messages.clone()
    }

    fn remove_history_last_message<S: Into<String>>(&mut self, history_id: S) {
        if let Some(history) = self.messages_history.as_mut() {
            history.pop_last_message_for_id(history_id);
        }
    }
}

#[cfg(all(feature = "chat-history", feature = "stream"))]
impl Ollama {
    async fn get_chat_messages_by_id_async(&mut self, id: String) -> Vec<ChatMessage> {
        // Clone the current chat messages to avoid borrowing issues
        // And not to add message to the history if the request fails
        self.messages_history_async
            .as_mut()
            .unwrap_or(&mut MessagesHistoryAsync::default())
            .messages_by_id
            .lock()
            .await
            .entry(id.clone())
            .or_default()
            .clone()
    }

    pub async fn store_chat_message_by_id_async(&mut self, id: String, message: ChatMessage) {
        if let Some(messages_history_async) = self.messages_history_async.as_mut() {
            messages_history_async.add_message(id, message).await;
        }
    }

    pub async fn send_chat_messages_with_history_stream(
        &mut self,
        mut request: ChatMessageRequest,
        id: String,
    ) -> crate::error::Result<ChatMessageResponseStream> {
        use tokio_stream::StreamExt;

        let (tx, mut rx) =
            tokio::sync::mpsc::unbounded_channel::<Result<ChatMessageResponse, ()>>(); // create a channel for sending and receiving messages

        let mut current_chat_messages = self.get_chat_messages_by_id_async(id.clone()).await;

        if let Some(messaeg) = request.messages.first() {
            current_chat_messages.push(messaeg.clone());
        }

        request.messages.clone_from(&current_chat_messages);

        let mut stream = self.send_chat_messages_stream(request.clone()).await?;

        let message_history_async = self.messages_history_async.clone();

        tokio::spawn(async move {
            let mut result = String::new();
            while let Some(res) = rx.recv().await {
                match res {
                    Ok(res) => {
                        if let Some(message) = res.message.clone() {
                            result += message.content.as_str();
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }

            if let Some(message_history_async) = message_history_async {
                message_history_async
                    .add_message(id.clone(), ChatMessage::assistant(result))
                    .await;
            } else {
                eprintln!("not using chat-history and stream features"); // this should not happen if the features are enabled
            }
        });

        let s = stream! {
            while let Some(res) = stream.next().await {
                match res {
                    Ok(res) => {
                        if let Err(e) = tx.send(Ok(res.clone())) {
                            eprintln!("Failed to send response: {}", e);
                        };
                        yield Ok(res);
                    }
                    Err(_) => {
                        yield Err(());
                    }
                }
            }
        };

        Ok(Box::pin(s))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
}
