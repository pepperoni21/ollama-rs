pub mod list_local;
pub mod show_info;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Model {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
}