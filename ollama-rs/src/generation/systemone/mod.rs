use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{error::OllamaError, Ollama};

pub mod request;

use request::SystemOneRequest;

impl Ollama {
    /// Answer choice, yes/no, and scoring questions with a local System One model.
    pub async fn system_one(
        &self,
        request: SystemOneRequest,
    ) -> crate::error::Result<SystemOneResponse> {
        let url = format!("{}v1/systemone", self.url_str());
        let builder = self.reqwest_client.post(url);

        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());

        let response = builder.json(&request).send().await?;

        if !response.status().is_success() {
            return Err(OllamaError::Other(
                response
                    .text()
                    .await
                    .unwrap_or_else(|error| error.to_string()),
            ));
        }

        Ok(response.json().await?)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemOneResponse {
    pub model: String,
    pub answers: BTreeMap<String, SystemOneAnswer>,
    pub usage: SystemOneUsage,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum SystemOneAnswer {
    Choice {
        choice: String,
        probabilities: BTreeMap<String, f64>,
        confidence: f64,
    },
    Noul {
        noul: f64,
    },
    Score {
        score: f64,
        legend: BTreeMap<String, String>,
        probabilities: BTreeMap<String, f64>,
        confidence: f64,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemOneUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}
