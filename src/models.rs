pub mod list_local;
pub mod show_info;
pub mod create;
pub mod copy;
pub mod delete;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Model {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelInfo {
    pub license: String,
    pub modelfile: String,
    pub parameters: String,
    pub template: String,
}