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

/// Store for messages history
impl MessagesHistory {
    /// Generate a MessagesHistory
    pub fn new(messages_number_limit: u16) -> Self {
        Self {
            messages_by_id: HashMap::new(),
            messages_number_limit: messages_number_limit.max(2),
        }
    }

    /// Add message for entry even no history exists for an entry
    pub fn add_message<S: Into<String>>(&mut self, entry_id: S, message: ChatMessage) {
        let messages = self.messages_by_id.entry(entry_id.into()).or_default();

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

    /// Get Option with list of ChatMessage
    pub fn get_messages(&self, entry_id: &str) -> Option<&Vec<ChatMessage>> {
        self.messages_by_id.get(entry_id)
    }

    /// Clear history for an entry
    pub fn clear_messages_for_id(&mut self, entry_id: &str) {
        self.messages_by_id.remove(entry_id);
    }

    /// Remove last message added in history
    pub fn pop_last_message_for_id<S: Into<String>>(&mut self, entry_id: S) {
        if let Some(messages) = self.messages_by_id.get_mut(&entry_id.into()) {
            messages.pop();
        }
    }

    /// Remove a whole history
    pub fn clear_all_messages(&mut self) {
        self.messages_by_id = HashMap::new();
    }
}

impl Ollama {
    /// Create default instance with chat history
    pub fn new_default_with_history(messages_number_limit: u16) -> Self {
        Self {
            messages_history: Some(std::sync::Arc::new(std::sync::RwLock::new(
                MessagesHistory::new(messages_number_limit),
            ))),
            ..Default::default()
        }
    }

    /// Create new instance with chat history
    ///
    /// # Panics
    ///
    /// Panics if the host is not a valid URL or if the URL cannot have a port.
    pub fn new_with_history(
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
    pub fn new_with_history_from_url(url: url::Url, messages_number_limit: u16) -> Self {
        Self {
            url,
            messages_history: Some(std::sync::Arc::new(std::sync::RwLock::new(
                MessagesHistory::new(messages_number_limit),
            ))),
            ..Default::default()
        }
    }

    #[inline]
    pub fn try_new_with_history(
        url: impl crate::IntoUrl,
        messages_number_limit: u16,
    ) -> Result<Self, url::ParseError> {
        Ok(Self {
            url: url.into_url()?,
            messages_history: Some(std::sync::Arc::new(std::sync::RwLock::new(
                MessagesHistory::new(messages_number_limit),
            ))),
            ..Default::default()
        })
    }

    /// Add AI's message to a history
    pub fn add_assistant_response<S: Into<String>>(&mut self, entry_id: S, message: S) {
        self.add_history_message(entry_id, ChatMessage::assistant(message.into()));
    }

    /// Add user's message to a history
    pub fn add_user_response<S: Into<String>>(&mut self, entry_id: S, message: S) {
        self.add_history_message(entry_id, ChatMessage::user(message.into()));
    }

    /// Set system prompt for chat history
    pub fn set_system_response<S: Into<String>>(&mut self, entry_id: S, message: S) {
        self.add_history_message(entry_id, ChatMessage::system(message.into()));
    }

    /// Helper for message add to history
    fn add_history_message<S: Into<String>>(&mut self, entry_id: S, message: ChatMessage) {
        if let Some(messages_history) = self.messages_history.as_mut() {
            messages_history
                .write()
                .unwrap()
                .add_message(entry_id, message);
        }
    }

    /// For tests purpose
    /// Getting list of messages in a history
    pub fn get_messages_history(&mut self, entry_id: &str) -> Option<Vec<ChatMessage>> {
        self.messages_history.clone().map(|message_history| {
            message_history
                .write()
                .unwrap()
                .get_messages(entry_id)
                .cloned()
        })?
    }
}
