pub mod models;
pub mod generation;
pub mod error;

pub struct Ollama {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) reqwest_client: reqwest::Client,
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
        format!("http://{}:{}", self.host, self.port)
    }
}

impl Default for Ollama {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 11434,
            reqwest_client: reqwest::Client::new(),
        }
    }
}