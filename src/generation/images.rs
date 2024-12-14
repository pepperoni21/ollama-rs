use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image(String);

impl Image {
    pub fn from_base64(base64: impl Into<String>) -> Self {
        Self(base64.into())
    }

    pub fn to_base64(&self) -> &str {
        &self.0
    }
}
