pub mod error;
pub mod generation;
#[cfg(feature = "chat-history")]
pub mod history;
pub mod models;

#[derive(Debug, Clone)]
pub struct Ollama {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) reqwest_client: reqwest::Client,
    #[cfg(feature = "chat-history")]
    pub(crate) messages_history: Option<history::MessagesHistory>,
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
