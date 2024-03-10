use generation::chat::{ChatMessage, MessagesHistory};

pub mod error;
pub mod generation;
pub mod models;

#[derive(Debug, Clone)]
pub struct Ollama {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) reqwest_client: reqwest::Client,
    #[cfg(feature = "chat-history")]
    pub(crate) messages_history: Option<MessagesHistory>,
}

impl Ollama {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            ..Default::default()
        }
    }

    /// Returns the http URI of the Ollama instance
    pub fn uri(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

}

#[cfg(feature = "chat-history")]
impl Ollama {
    /// Create default instance with chat history
    pub fn new_default_with_history(messages_number_limit: u16) -> Self {
        Self {
            messages_history: Some(MessagesHistory::new(messages_number_limit)),
            ..Default::default()
        }
    }
    
    /// Create new instance with chat history 
    pub fn new_with_history(host: String, port: u16, messages_number_limit: u16) -> Self {
        Self {
            host,
            port,
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

impl Default for Ollama {
    /// Returns a default Ollama instance with the host set to `http://127.0.0.1:11434`.
    fn default() -> Self {
        Self {
            host: "http://127.0.0.1".to_string(),
            port: 11434,
            reqwest_client: reqwest::Client::new(),
            #[cfg(feature = "chat-history")]
            messages_history: None,
        }
    }
}
