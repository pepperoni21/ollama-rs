use serde::Deserialize;

use crate::Ollama;

use self::request::GenerateEmbeddingsRequest;

pub mod request;

impl Ollama {
    /// Generate embeddings from a model
    /// * `model_name` - Name of model to generate embeddings from
    /// * `prompt` - Prompt to generate embeddings for
    pub async fn generate_embeddings(
        &self,
        request: GenerateEmbeddingsRequest,
    ) -> crate::error::Result<GenerateEmbeddingsResponse> {
        let url = format!("{}api/embed", self.url_str());
        let builder = self.reqwest_client.post(url);

        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());

        crate::map_response::<GenerateEmbeddingsResponse>(builder.json(&request).send().await?)
            .await
    }
}

/// An embeddings generation response from Ollama.
#[derive(Debug, Deserialize, Clone)]
pub struct GenerateEmbeddingsResponse {
    #[allow(dead_code)]
    pub embeddings: Vec<Vec<f32>>,
}
