use generation::chat::{ChatMessage, MessagesHistory};

pub mod error;
pub mod generation;
pub mod models;

#[derive(Debug, Clone)]
pub struct Ollama {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) reqwest_client: reqwest::Client,
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

    pub fn new_default_with_history(messages_number_limit: u16) -> Self {
        Self {
            messages_history: Some(MessagesHistory::new(messages_number_limit)),
            ..Default::default()
        }
    }

    pub fn new_with_history(host: String, port: u16, messages_number_limit: u16) -> Self {
        Self {
            host,
            port,
            messages_history: Some(MessagesHistory::new(messages_number_limit)),
            ..Default::default()
        }
    }

    /// Returns the http URI of the Ollama instance
    pub fn uri(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Add message to a history
    pub fn add_assistant_response(&mut self, entry_id: String, message: String) {
        if let Some(messages_history) = self.messages_history.as_mut() {
            messages_history.add_message(entry_id, ChatMessage::assistant(message));
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
            messages_history: None,
        }
    }
}
