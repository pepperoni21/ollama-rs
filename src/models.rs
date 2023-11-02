mod list_local;

pub use list_local::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Model {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
}