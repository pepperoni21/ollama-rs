pub mod copy;
pub mod create;
pub mod delete;
pub mod list_local;
pub mod pull;
pub mod push;
pub mod show_info;

use serde::Deserialize;

/// A local model pulled from Ollama.
#[derive(Debug, Clone, Deserialize)]
pub struct LocalModel {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
}

/// A model's info.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelInfo {
    pub license: String,
    pub modelfile: String,
    pub parameters: String,
    pub template: String,
}
