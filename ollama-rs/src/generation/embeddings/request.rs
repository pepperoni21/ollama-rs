use serde::{Deserialize, Serialize, Serializer};

use crate::{generation::parameters::KeepAlive, models::ModelOptions};

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum EmbeddingsInput {
    Single(String),
    Multiple(Vec<String>),
}

impl Default for EmbeddingsInput {
    fn default() -> Self {
        Self::Single(String::default())
    }
}

impl From<String> for EmbeddingsInput {
    fn from(s: String) -> Self {
        Self::Single(s)
    }
}

impl From<&str> for EmbeddingsInput {
    fn from(s: &str) -> Self {
        Self::Single(s.to_string())
    }
}

impl From<Vec<String>> for EmbeddingsInput {
    fn from(v: Vec<String>) -> Self {
        Self::Multiple(v)
    }
}

impl From<Vec<&str>> for EmbeddingsInput {
    fn from(v: Vec<&str>) -> Self {
        Self::Multiple(v.iter().map(|s| s.to_string()).collect())
    }
}

impl Serialize for EmbeddingsInput {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            EmbeddingsInput::Single(s) => s.serialize(serializer),
            EmbeddingsInput::Multiple(v) => v.serialize(serializer),
        }
    }
}

/// An embeddings generation request to Ollama.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GenerateEmbeddingsRequest {
    #[serde(rename = "model")]
    pub model_name: String,
    pub input: EmbeddingsInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncate: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ModelOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<KeepAlive>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,
}

impl GenerateEmbeddingsRequest {
    pub fn new(model_name: String, input: EmbeddingsInput) -> Self {
        Self {
            model_name,
            input,
            ..Default::default()
        }
    }

    pub fn options(mut self, options: ModelOptions) -> Self {
        self.options = Some(options);
        self
    }

    pub fn keep_alive(mut self, keep_alive: KeepAlive) -> Self {
        self.keep_alive = Some(keep_alive);
        self
    }

    pub fn truncate(mut self, truncate: bool) -> Self {
        self.truncate = Some(truncate);
        self
    }

    pub fn dimensions(mut self, dimensions: u32) -> Self {
        self.dimensions = Some(dimensions);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::generation::embeddings::request::{EmbeddingsInput, GenerateEmbeddingsRequest};

    #[test]
    fn serde_embedding_request_single() {
        let request = GenerateEmbeddingsRequest::new(
            "test".to_string(),
            EmbeddingsInput::Single("test".to_string()),
        );
        let json = serde_json::to_vec(&request).unwrap();
        let parsed_request: GenerateEmbeddingsRequest = serde_json::from_slice(&json).unwrap();
        assert_eq!(request.model_name, parsed_request.model_name);
        assert_eq!(request.input, parsed_request.input);
        assert_eq!(request.truncate, parsed_request.truncate);
        assert_eq!(request.keep_alive, parsed_request.keep_alive);
    }

    #[test]
    fn serde_embedding_request_multiple() {
        let request = GenerateEmbeddingsRequest::new(
            "test".to_string(),
            EmbeddingsInput::Multiple(vec!["test".to_string()]),
        );
        let json = serde_json::to_vec(&request).unwrap();
        let parsed_request: GenerateEmbeddingsRequest = serde_json::from_slice(&json).unwrap();
        assert_eq!(request.model_name, parsed_request.model_name);
        assert_eq!(request.input, parsed_request.input);
        assert_eq!(request.truncate, parsed_request.truncate);
        assert_eq!(request.keep_alive, parsed_request.keep_alive);
    }
}
