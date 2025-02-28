use serde::{Deserialize, Serialize};

use crate::{error::OllamaError, Ollama};

/// A stream of `CreateModelStatus` objects
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
pub type CreateModelStatusStream = std::pin::Pin<
    Box<dyn tokio_stream::Stream<Item = crate::error::Result<CreateModelStatus>> + Send>,
>;

impl Ollama {
    #[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
    #[cfg(feature = "stream")]
    /// Create a model with streaming, meaning that each new status will be streamed.
    pub async fn create_model_stream(
        &self,
        mut request: CreateModelRequest,
    ) -> crate::error::Result<CreateModelStatusStream> {
        use tokio_stream::StreamExt;

        use crate::error::OllamaError;

        request.stream = true;

        let url = format!("{}api/create", self.url_str());
        let serialized = serde_json::to_string(&request)?;
        let builder = self.reqwest_client.post(url);

        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());

        let res = builder.body(serialized).send().await?;

        if !res.status().is_success() {
            return Err(OllamaError::Other(res.text().await?));
        }

        let stream = Box::new(res.bytes_stream().map(|res| match res {
            Ok(bytes) => {
                let res = serde_json::from_slice::<CreateModelStatus>(&bytes);
                match res {
                    Ok(res) => Ok(res),
                    Err(e) => {
                        let err =
                            serde_json::from_slice::<crate::error::InternalOllamaError>(&bytes);
                        match err {
                            Ok(err) => Err(OllamaError::InternalError(err)),
                            Err(_) => Err(OllamaError::from(e)),
                        }
                    }
                }
            }
            Err(e) => Err(OllamaError::Other(format!(
                "Failed to read response: {}",
                e
            ))),
        }));

        Ok(std::pin::Pin::from(stream))
    }

    /// Create a model with a single response, only the final status will be returned.
    pub async fn create_model(
        &self,
        request: CreateModelRequest,
    ) -> crate::error::Result<CreateModelStatus> {
        let url = format!("{}api/create", self.url_str());
        let serialized = serde_json::to_string(&request)?;
        let builder = self.reqwest_client.post(url);

        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());

        let res = builder.body(serialized).send().await?;

        if !res.status().is_success() {
            return Err(OllamaError::Other(res.text().await?));
        }

        let res = res.bytes().await?;
        let res = serde_json::from_slice::<CreateModelStatus>(&res)?;

        Ok(res)
    }
}

/// A create model request to Ollama.
#[derive(Serialize)]
pub struct CreateModelRequest {
    #[serde(rename = "name")]
    model_name: String,
    path: Option<String>,
    modelfile: Option<String>,
    stream: bool,
}

impl CreateModelRequest {
    /// Create a model described in the Modelfile at `path`.
    pub fn path(model_name: String, path: String) -> Self {
        Self {
            model_name,
            path: Some(path),
            modelfile: None,
            stream: false,
        }
    }

    /// Create a model described by the Modelfile contents passed to `modelfile`.
    pub fn modelfile(model_name: String, modelfile: String) -> Self {
        Self {
            model_name,
            path: None,
            modelfile: Some(modelfile),
            stream: false,
        }
    }
}

/// A create model status response from Ollama.
#[derive(Deserialize, Debug)]
pub struct CreateModelStatus {
    #[serde(rename = "status")]
    pub message: String,
}
