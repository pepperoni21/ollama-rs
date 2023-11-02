use std::fmt::{Display, Debug};

pub type Result<T> = std::result::Result<T, OllamaError>;

pub struct OllamaError {
    pub(crate) message: String,
}

impl Display for OllamaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "An error occurred with ollama-rs: {}", self.message)
    }
}

impl Debug for OllamaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OllamaError").field("message", &self.message).finish()
    }
}

impl From<String> for OllamaError {
    fn from(message: String) -> Self {
        Self { message }
    }
}