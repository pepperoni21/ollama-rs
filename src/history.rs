use std::collections::HashMap;

use crate::{
    generation::chat::{ChatMessage, MessageRole},
    Ollama,
};

#[derive(Debug, Clone, Default)]
pub struct MessagesHistory {
    pub(crate) messages_by_id: HashMap<String, Vec<ChatMessage>>,
    pub(crate) messages_number_limit: u16,
}

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

impl Ollama {
    /// Create default instance with chat history
    pub fn new_default_with_history(messages_number_limit: u16) -> Self {
        Self {
            messages_history: Some(MessagesHistory::new(messages_number_limit)),
            ..Default::default()
        }
    }

    /// Create new instance with chat history
    ///
    /// # Panics
    ///
    /// Panics if the host is not a valid URL or if the URL cannot have a port.
    pub fn new_with_history(
        host: impl Into<url::Url>,
        port: u16,
        messages_number_limit: u16,
    ) -> Self {
        let mut url = host.into();
        url.set_port(Some(port)).unwrap();
        Self::new_with_history_from_url(url, messages_number_limit)
    }

    /// Create new instance with chat history from a [`url::Url`].
    #[inline]
    pub fn new_with_history_from_url(url: url::Url, messages_number_limit: u16) -> Self {
        Self {
            url,
            messages_history: Some(MessagesHistory::new(messages_number_limit)),
            ..Default::default()
        }
    }

    /// Add AI's message to a history
    pub fn add_assistant_response(&mut self, entry_id: String, message: String) {
        if let Some(messages_history) = self.messages_history.as_mut() {
            messages_history.add_message(entry_id, ChatMessage::assistant(message));
        }
    }

    /// Add user's message to a history
    pub fn add_user_response(&mut self, entry_id: String, message: String) {
        if let Some(messages_history) = self.messages_history.as_mut() {
            messages_history.add_message(entry_id, ChatMessage::user(message));
        }
    }

    /// Set system prompt for chat history
    pub fn set_system_response(&mut self, entry_id: String, message: String) {
        if let Some(messages_history) = self.messages_history.as_mut() {
            messages_history.add_message(entry_id, ChatMessage::system(message));
        }
    }

    /// For tests purpose
    /// Getting list of messages in a history
    pub fn get_messages_history(&mut self, entry_id: String) -> Option<&Vec<ChatMessage>> {
        if let Some(messages_history) = self.messages_history.as_mut() {
            messages_history.messages_by_id.get(&entry_id)
        } else {
            None
        }
    }
}
