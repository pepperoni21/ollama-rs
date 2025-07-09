use serde::{Deserialize, Serialize};

use crate::{error::OllamaError, Ollama};

/// A stream of `PushModelStatus` objects.
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
pub type PushModelStatusStream = std::pin::Pin<
    Box<dyn tokio_stream::Stream<Item = crate::error::Result<PushModelStatus>> + Send>,
>;

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
        use crate::error::OllamaError;
        use tokio_stream::StreamExt;

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

        if !res.status().is_success() {
            return Err(OllamaError::Other(res.text().await?));
        }

        let stream = Box::new(res.bytes_stream().map(|res| match res {
            Ok(bytes) => {
                let res = serde_json::from_slice::<PushModelStatus>(&bytes);
                match res {
                    Ok(res) => Ok(res),
                    Err(e) => {
                        let err =
                            serde_json::from_slice::<crate::error::InternalOllamaError>(&bytes);
                        match err {
                            Ok(err) => Err(OllamaError::InternalError(err)),
                            Err(_) => Err(e.into()),
                        }
                    }
                }
            }
            Err(e) => Err(OllamaError::Other(format!("Failed to read response: {e}"))),
        }));

        Ok(std::pin::Pin::from(stream))
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

        let url = format!("{}api/push", self.url_str());
        let builder = self.reqwest_client.post(url);

        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());

        let res = builder.json(&request).send().await?;

        if !res.status().is_success() {
            return Err(OllamaError::Other(res.text().await?));
        }

        let res = res.bytes().await?;
        let res = serde_json::from_slice::<PushModelStatus>(&res)?;

        Ok(res)
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
