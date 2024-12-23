use serde::Deserialize;

use crate::{error::OllamaError, Ollama};

use super::LocalModel;

impl Ollama {
    pub async fn list_local_models(&self) -> crate::error::Result<Vec<LocalModel>> {
        let url = format!("{}api/tags", self.url_str());
        let builder = self.reqwest_client.get(url);

        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());

        let res = builder.send().await?;

        if !res.status().is_success() {
            return Err(OllamaError::Other(res.text().await?));
        }

        let res = res.bytes().await?;
        let res = serde_json::from_slice::<ListLocalModelsResponse>(&res)?;

        Ok(res.models)
    }
}

/// A response from Ollama containing a list of local models.
#[derive(Debug, Clone, Deserialize)]
struct ListLocalModelsResponse {
    models: Vec<LocalModel>,
}
