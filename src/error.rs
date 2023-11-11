use std::{
    error::Error,
    fmt::{Debug, Display},
};

use serde::Deserialize;

pub type Result<T> = std::result::Result<T, OllamaError>;

#[derive(Deserialize)]
pub struct OllamaError {
    #[serde(rename = "error")]
    pub(crate) message: String,
}

impl Display for OllamaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "An error occurred with ollama-rs: {}", self.message)
    }
}

impl Debug for OllamaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ollama error: {}", self.message)
    }
}

impl Error for OllamaError {}

impl From<String> for OllamaError {
    fn from(message: String) -> Self {
        Self { message }
    }
}
