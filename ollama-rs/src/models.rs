/// Modules related to model operations.
///
/// These modules provide functionality for copying, creating, deleting,
/// listing, pulling, pushing, and showing information about models.
pub mod copy;
pub mod create;
pub mod delete;
pub mod list_local;
pub mod pull;
pub mod push;
pub mod show_info;

use serde::{Deserialize, Serialize};

/// Represents a local model pulled from Ollama.
///
/// This struct contains information about a model that has been pulled
/// from the Ollama service, including its name, modification date, and size.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModel {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
}

/// Represents information about a model.
///
/// This struct contains various fields that describe a model's attributes,
/// such as its license, file, parameters, and template.
/// Some fields may be empty if the model does not have them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    #[serde(default = "String::new")]
    pub license: String,
    #[serde(default = "String::new")]
    pub modelfile: String,
    #[serde(default = "String::new")]
    pub parameters: String,
    #[serde(default = "String::new")]
    pub template: String,
}
