use serde::{Deserialize, Serialize};

use crate::{error::OllamaError, Ollama};

/// A stream of `PullModelStatus` objects.
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
pub type PullModelStatusStream = std::pin::Pin<
    Box<dyn tokio_stream::Stream<Item = crate::error::Result<PullModelStatus>> + Send>,
>;

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
        use tokio_stream::StreamExt;

        use crate::error::{InternalOllamaError, OllamaError};

        let request = PullModelRequest {
            model_name,
            allow_insecure,
            stream: true,
        };

        let url = format!("{}api/pull", self.url_str());
        let builder = self.reqwest_client.post(url);

        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());

        let res = builder.json(&request).send().await?;

        if !res.status().is_success() {
            return Err(OllamaError::Other(res.text().await?));
        }

        let stream = Box::new(res.bytes_stream().map(|res| match res {
            Ok(bytes) => {
                let res = serde_json::from_slice::<PullModelStatus>(&bytes);
                match res {
                    Ok(res) => Ok(res),
                    Err(e) => {
                        let err = serde_json::from_slice::<InternalOllamaError>(&bytes);
                        match err {
                            Ok(err) => Err(OllamaError::InternalError(err)),
                            Err(_) => Err(e.into()),
                        }
                    }
                }
            }
            Err(e) => Err(e.into()),
        }));

        Ok(std::pin::Pin::from(stream))
    }

    /// Pull a model with a single response, only the final status will be returned.
    /// - `model_name` - The name of the model to pull.
    /// - `allow_insecure` - Allow insecure connections to the library. Only use this if you are pulling from your own library during development.
    pub async fn pull_model(
        &self,
        model_name: String,
        allow_insecure: bool,
    ) -> crate::error::Result<PullModelStatus> {
        let request = PullModelRequest {
            model_name,
            allow_insecure,
            stream: false,
        };

        let url = format!("{}api/pull", self.url_str());
        let builder = self.reqwest_client.post(url);

        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());

        let res = builder.json(&request).send().await?;

        if !res.status().is_success() {
            return Err(OllamaError::Other(res.text().await?));
        }

        let res = res.bytes().await?;
        let res = serde_json::from_slice::<PullModelStatus>(&res)?;

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
