use serde::Deserialize;

use crate::Ollama;

use super::LocalModel;

impl Ollama {
    pub async fn list_local_models(&self) -> crate::error::Result<Vec<LocalModel>> {
        let url = format!("{}api/tags", self.url_str());
        let builder = self.reqwest_client.get(url);

        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());

        crate::map_response::<ListLocalModelsResponse>(builder.send().await?)
            .await
            .map(|a| a.models)
    }
}

/// A response from Ollama containing a list of local models.
#[derive(Debug, Clone, Deserialize)]
struct ListLocalModelsResponse {
    models: Vec<LocalModel>,
}
