use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Model {
    pub name: String,
    pub modified_at: Instant,
    pub size: u64,
}