use serde::Serialize;

use crate::Ollama;

use super::ModelInfo;

impl Ollama {
    /// Show details about a model including modelfile, template, parameters, license, and system prompt.
    pub async fn show_model_info(&self, model_name: String) -> crate::error::Result<ModelInfo> {
        let url = format!("{}api/show", self.url_str());
        let builder = self.reqwest_client.post(url);

        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());
        let request = ModelInfoRequest { model_name };
        crate::map_response(builder.json(&request).send().await?).await
    }
}

/// A show model info request to Ollama.
#[derive(Serialize)]
struct ModelInfoRequest {
    #[serde(rename = "name")]
    model_name: String,
}
