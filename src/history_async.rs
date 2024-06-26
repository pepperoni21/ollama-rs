use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::{
    generation::chat::{ChatMessage, MessageRole},
    Ollama,
};

#[derive(Debug, Clone, Default)]
pub struct MessagesHistoryAsync {
    pub(crate) messages_by_id: Arc<Mutex<HashMap<String, Vec<ChatMessage>>>>,
    pub(crate) messages_number_limit: u16,
}

impl MessagesHistoryAsync {
    pub fn new(messages_number_limit: u16) -> Self {
        Self {
            messages_by_id: Arc::new(Mutex::new(HashMap::new())),
            messages_number_limit: messages_number_limit.max(2),
        }
    }

    pub async fn add_message(&mut self, entry_id: String, message: ChatMessage) {
        let mut messages_lock = self.messages_by_id.lock().await;
        let messages = messages_lock.entry(entry_id).or_default();

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

    pub async fn get_messages(&self, entry_id: &str) -> Option<Vec<ChatMessage>> {
        let messages_lock = self.messages_by_id.lock().await;
        messages_lock.get(entry_id).cloned()
    }

    pub async fn clear_messages(&mut self, entry_id: &str) {
        let mut messages_lock = self.messages_by_id.lock().await;
        messages_lock.remove(entry_id);
    }
}

impl Ollama {
    /// Create default instance with chat history
    pub fn new_default_with_history_async(messages_number_limit: u16) -> Self {
        Self {
            messages_history_async: Some(MessagesHistoryAsync::new(messages_number_limit)),
            ..Default::default()
        }
    }

    /// Create new instance with chat history
    ///
    /// # Panics
    ///
    /// Panics if the host is not a valid URL or if the URL cannot have a port.
    pub fn new_with_history_async(
        host: impl crate::IntoUrl,
        port: u16,
        messages_number_limit: u16,
    ) -> Self {
        let mut url = host.into_url().unwrap();
        url.set_port(Some(port)).unwrap();
        Self::new_with_history_from_url(url, messages_number_limit)
    }

    /// Create new instance with chat history from a [`url::Url`].
    #[inline]
    pub fn new_with_history_from_url_async(url: url::Url, messages_number_limit: u16) -> Self {
        Self {
            url,
            messages_history_async: Some(MessagesHistoryAsync::new(messages_number_limit)),
            ..Default::default()
        }
    }

    #[inline]
    pub fn try_new_with_history_async(
        url: impl crate::IntoUrl,
        messages_number_limit: u16,
    ) -> Result<Self, url::ParseError> {
        Ok(Self {
            url: url.into_url()?,
            messages_history_async: Some(MessagesHistoryAsync::new(messages_number_limit)),
            ..Default::default()
        })
    }

    /// Add AI's message to a history
    pub async fn add_assistant_response_async(&mut self, entry_id: String, message: String) {
        if let Some(messages_history) = self.messages_history_async.as_mut() {
            messages_history
                .add_message(entry_id, ChatMessage::assistant(message))
                .await;
        }
    }

    /// Add user's message to a history
    pub async fn add_user_response_async(&mut self, entry_id: String, message: String) {
        if let Some(messages_history) = self.messages_history_async.as_mut() {
            messages_history
                .add_message(entry_id, ChatMessage::user(message))
                .await;
        }
    }

    /// Set system prompt for chat history
    pub async fn set_system_response_async(&mut self, entry_id: String, message: String) {
        if let Some(messages_history) = self.messages_history_async.as_mut() {
            messages_history
                .add_message(entry_id, ChatMessage::system(message))
                .await;
        }
    }

    /// For tests purpose
    /// Getting list of messages in a history
    pub async fn get_messages_history_async(
        &mut self,
        entry_id: String,
    ) -> Option<Vec<ChatMessage>> {
        if let Some(messages_history_async) = self.messages_history_async.as_mut() {
            messages_history_async.get_messages(&entry_id).await
        } else {
            None
        }
    }
}
