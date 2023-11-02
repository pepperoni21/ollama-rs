use serde::Serialize;

use crate::Ollama;

impl Ollama {
    /// Copy a model. Creates a model with another name from an existing model.
    pub async fn copy_model(&self, source: String, destination: String) -> crate::error::Result<()> {
        let request = CopyModelRequest {
            source,
            destination,
        };

        let uri = format!("{}/api/copy", self.uri());
        let serialized = serde_json::to_string(&request).map_err(|e| e.to_string())?;
        let res = self.reqwest_client.post(uri)
            .body(serialized)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        
        if res.status().is_success() {
            Ok(())
        } else {
            Err(res.text().await.unwrap_or_else(|e| e.to_string()).into())
        }
    }
}

#[derive(Serialize)]
struct CopyModelRequest {
    source: String,
    destination: String,
}