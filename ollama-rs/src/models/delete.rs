use serde::Serialize;

use crate::{error::OllamaError, Ollama};

impl Ollama {
    /// Delete a model and its data.
    pub async fn delete_model(&self, model_name: String) -> crate::error::Result<()> {
        let request = DeleteModelRequest { model_name };

        let url = format!("{}api/delete", self.url_str());
        let builder = self.reqwest_client.delete(url);

        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());

        let res = builder.json(&request).send().await?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(OllamaError::Other(res.text().await?))
        }
    }
}

/// A delete model request to Ollama.
#[derive(Serialize)]
struct DeleteModelRequest {
    #[serde(rename = "name")]
    model_name: String,
}
