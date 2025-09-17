use serde::Serialize;

use crate::Ollama;

impl Ollama {
    /// Copy a model. Creates a model with another name from an existing model.
    pub async fn copy_model(
        &self,
        source: String,
        destination: String,
    ) -> crate::error::Result<()> {
        let request = CopyModelRequest {
            source,
            destination,
        };

        let url = format!("{}api/copy", self.url_str());
        let builder = self.reqwest_client.post(url);

        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());

        crate::map_empty_response(builder.json(&request).send().await?).await
    }
}

/// A copy model request to Ollama.
#[derive(Serialize)]
struct CopyModelRequest {
    source: String,
    destination: String,
}
