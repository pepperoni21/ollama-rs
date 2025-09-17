use serde::{Deserialize, Serialize};

use crate::{error::OllamaError, Ollama};

/// A stream of `PullModelStatus` objects.
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
pub type PullModelStatusStream =
    futures_util::stream::BoxStream<'static, crate::error::Result<PullModelStatus>>;

impl Ollama {
    #[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
    #[cfg(feature = "stream")]
    /// Pull a model with streaming, meaning that each new status will be streamed.
    /// - `model_name` - The name of the model to pull.
    /// - `allow_insecure` - Allow insecure connections to the library. Only use this if you are pulling from your own library during development.
    pub async fn pull_model_stream(
        &self,
        model_name: String,
        allow_insecure: bool,
    ) -> crate::error::Result<PullModelStatusStream> {
        crate::stream::map_response(
            self.send_pull_model_request(PullModelRequest {
                model_name,
                allow_insecure,
                stream: true,
            })
            .await?,
        )
        .await
    }

    /// Pull a model with a single response, only the final status will be returned.
    /// - `model_name` - The name of the model to pull.
    /// - `allow_insecure` - Allow insecure connections to the library. Only use this if you are pulling from your own library during development.
    pub async fn pull_model(
        &self,
        model_name: String,
        allow_insecure: bool,
    ) -> crate::error::Result<PullModelStatus> {
        crate::map_response(
            self.send_pull_model_request(PullModelRequest {
                model_name,
                allow_insecure,
                stream: false,
            })
            .await?,
        )
        .await
    }

    async fn send_pull_model_request(
        &self,
        request: PullModelRequest,
    ) -> Result<reqwest::Response, OllamaError> {
        let url = format!("{}api/pull", self.url_str());
        let builder = self.reqwest_client.post(url);
        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());
        let res = builder.json(&request).send().await?;
        Ok(res)
    }
}

/// A pull model request to Ollama.
#[derive(Debug, Clone, Serialize)]
struct PullModelRequest {
    #[serde(rename = "name")]
    model_name: String,
    #[serde(rename = "insecure")]
    allow_insecure: bool,
    stream: bool,
}

/// A pull model status response from Ollama.
#[derive(Debug, Clone, Deserialize)]
pub struct PullModelStatus {
    #[serde(rename = "status")]
    pub message: String,
    pub digest: Option<String>,
    pub total: Option<u64>,
    pub completed: Option<u64>,
}
