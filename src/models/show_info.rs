use serde::Serialize;

use crate::Ollama;

use super::ModelInfo;

impl Ollama {
    /// Show details about a model including modelfile, template, parameters, license, and system prompt.
    pub async fn show_model_info(&self, model_name: String) -> crate::error::Result<ModelInfo> {
        let uri = format!("{}/api/show", self.uri());
        let serialized =
            serde_json::to_string(&ModelInfoRequest { model_name }).map_err(|e| e.to_string())?;
        let res = self
            .reqwest_client
            .post(uri)
            .body(serialized)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(res.text().await.unwrap_or_else(|e| e.to_string()).into());
        }

        let res = res.bytes().await.map_err(|e| e.to_string())?;
        let res = serde_json::from_slice::<ModelInfo>(&res).map_err(|e| e.to_string())?;

        Ok(res)
    }
}

/// A show model info request to Ollama.
#[derive(Serialize)]
struct ModelInfoRequest {
    #[serde(rename = "name")]
    model_name: String,
}
