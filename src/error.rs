use serde::Deserialize;
use thiserror::Error;

/// A result type for ollama-rs.
pub type Result<T> = std::result::Result<T, OllamaError>;

/// An error type for ollama-rs.
#[derive(Error, Debug)]
pub enum OllamaError {
    #[error("Error calling tool")]
    ToolCallError(#[from] ToolCallError),
    #[error("Ollama JSON error")]
    JsonError(#[from] serde_json::Error),
    #[error("Reqwest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Internal Ollama error")]
    InternalError(InternalOllamaError),
    #[error("Error in Ollama")]
    Other(String),
}

#[derive(Deserialize, Debug)]
pub struct InternalOllamaError {
    #[serde(rename = "error")]
    pub(crate) message: String,
}

#[derive(Error, Debug)]
pub enum ToolCallError {
    #[error("Ollama attempted to call a tool with a name we do not recognize")]
    UnknownToolName,
    #[error("Could not convert tool arguments from Ollamainto what the tool expected")]
    InvalidToolArguments(#[from] serde_json::Error),
}
