use serde::{Serialize, Serializer};

use crate::generation::{options::GenerationOptions, parameters::KeepAlive};

#[derive(Debug)]
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

impl Serialize for EmbeddingsInput {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            EmbeddingsInput::Single(s) => s.serialize(serializer),
            EmbeddingsInput::Multiple(v) => v.serialize(serializer),
        }
    }
}

/// An embeddings generation request to Ollama.
#[derive(Debug, Serialize, Default)]
pub struct GenerateEmbeddingsRequest {
    #[serde(rename = "model")]
    model_name: String,
    input: EmbeddingsInput,
    truncate: Option<bool>,
    options: Option<GenerationOptions>,
    keep_alive: Option<KeepAlive>,
}

impl GenerateEmbeddingsRequest {
    pub fn new(model_name: String, input: EmbeddingsInput) -> Self {
        Self {
            model_name,
            input,
            ..Default::default()
        }
    }

    pub fn options(mut self, options: GenerationOptions) -> Self {
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
}
