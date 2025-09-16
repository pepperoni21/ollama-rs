use serde::{Deserialize, Serialize};

use crate::{error::OllamaError, Ollama};

/// A stream of `PushModelStatus` objects.
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
pub type PushModelStatusStream =
    futures_util::stream::BoxStream<'static, crate::error::Result<PushModelStatus>>;

impl Ollama {
    #[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
    #[cfg(feature = "stream")]
    /// Upload a model to a model library. Requires registering for ollama.ai and adding a public key first.
    /// Push a model with streaming, meaning that each new status will be streamed.
    /// - `model_name` - The name of the model to push in the form of `<namespace>/<model>:<tag>`.
    /// - `allow_insecure` - Allow insecure connections to the library. Only use this if you are pushing to your library during development.
    pub async fn push_model_stream(
        &self,
        model_name: String,
        allow_insecure: bool,
    ) -> crate::error::Result<PushModelStatusStream> {
        let request = PushModelRequest {
            model_name,
            allow_insecure,
            stream: true,
        };

        let url = format!("{}api/push", self.url_str());
        let builder = self.reqwest_client.post(url);

        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());

        let res = builder.json(&request).send().await?;
        crate::stream::map_response(res).await
    }

    /// Upload a model to a model library. Requires registering for ollama.ai and adding a public key first.
    /// Push a model with a single response, only the final status will be returned.
    /// - `model_name` - The name of the model to push in the form of `<namespace>/<model>:<tag>`.
    /// - `allow_insecure` - Allow insecure connections to the library. Only use this if you are pushing to your library during development.
    pub async fn push_model(
        &self,
        model_name: String,
        allow_insecure: bool,
    ) -> crate::error::Result<PushModelStatus> {
        let request = PushModelRequest {
            model_name,
            allow_insecure,
            stream: false,
        };

        let res = self.push_model_request(request).await?;
        if res.status().is_success() {
            let bytes = res.bytes().await?;
            Ok(serde_json::from_slice::<PushModelStatus>(&bytes)?)
        } else {
            Err(OllamaError::Other(res.text().await?))
        }
    }

    async fn push_model_request(
        &self,
        request: PushModelRequest,
    ) -> Result<reqwest::Response, OllamaError> {
        let url = format!("{}api/push", self.url_str());
        let builder = self.reqwest_client.post(url);
        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());
        Ok(builder.json(&request).send().await?)
    }
}

/// A push model request to Ollama.
#[derive(Debug, Clone, Serialize)]
struct PushModelRequest {
    #[serde(rename = "name")]
    model_name: String,
    #[serde(rename = "insecure")]
    allow_insecure: bool,
    stream: bool,
}

/// A push model status response from Ollama.
#[derive(Debug, Clone, Deserialize)]
pub struct PushModelStatus {
    #[serde(rename = "status")]
    pub message: String,
    pub digest: Option<String>,
    pub total: Option<u64>,
}
