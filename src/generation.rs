use std::pin::Pin;

use serde::{Serialize, Deserialize};
use tokio_stream::{Stream, StreamExt};

use crate::Ollama;

use self::request::GenerationRequest;

pub mod options;
pub mod request;

impl Ollama {
    pub async fn generate_stream(&self, request: GenerationRequest) -> Result<GenerationResponseStream, String> {
        let mut request = request;
        request.stream = true;

        let uri = format!("{}/api/generate", self.uri());
        let serialized = serde_json::to_string(&request).map_err(|e| e.to_string())?;
        let res = self.reqwest_client.post(uri)
            .body(serialized)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(res.text().await.unwrap_or_else(|e| e.to_string()));
        }

        let stream = Box::new(res.bytes_stream().map(|res| {
            match res {
                Ok(bytes) => {
                    let res = serde_json::from_slice::<GenerationResponse>(&bytes);
                    match res {
                        Ok(res) => Ok(res),
                        Err(e) => {
                            eprintln!("Failed to deserialize response: {}", e);
                            Err(())
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read response: {}", e);
                    Err(())
                }
            }
        }));

        Ok(Pin::from(stream))
    }

    pub async fn generate(&self, request: GenerationRequest) -> Result<GenerationResponse, String> {
        let mut request = request;
        request.stream = false;

        let uri = format!("{}/api/generate", self.uri());
        let serialized = serde_json::to_string(&request).map_err(|e| e.to_string())?;
        let res = self.reqwest_client.post(uri)
            .body(serialized)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(res.text().await.unwrap_or_else(|e| e.to_string()));
        }

        let res = res.bytes().await.map_err(|e| e.to_string())?;
        let res = serde_json::from_slice::<GenerationResponse>(&res).map_err(|e| e.to_string())?;

        Ok(res)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationContext(Vec<i32>);

#[derive(Debug, Clone, Deserialize)]
pub struct GenerationResponse {
    pub model: String,
    pub created_at: String,
    pub response: String,
    pub done: bool,
    #[serde(flatten)]
    pub final_data: Option<GenerationFinalResponseData>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerationFinalResponseData {
    pub context: GenerationContext,
    pub total_duration: u64,
    pub load_duration: u64,
    pub prompt_eval_count: u16,
    pub prompt_eval_duration: u64,
    pub eval_count: u16,
    pub eval_duration: u64,
}

pub type GenerationResponseStream = Pin<Box<dyn Stream<Item = Result<GenerationResponse, ()>>>>;