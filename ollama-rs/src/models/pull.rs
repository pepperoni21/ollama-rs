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
        use crate::error::{InternalOllamaError, OllamaError};
        use tokio_stream::StreamExt;

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

        let mut stream = res.bytes_stream();

        // Use async-stream to create a generator
        let stream = async_stream::try_stream! {
            let mut buffer = Vec::new();

            while let Some(chunk) = stream.next().await {
                let bytes = chunk.map_err(|e| OllamaError::Other(e.to_string()))?;
                buffer.extend_from_slice(&bytes);

                // Process all complete lines in the buffer
                while let Some(i) = buffer.iter().position(|&b| b == b'\n') {
                    // Split the buffer at the newline:
                    // 'line' gets the data up to and including \n
                    // 'buffer' keeps the rest
                    let line_bytes: Vec<u8> = buffer.drain(..=i).collect();

                    // Slice off the newline for parsing
                    let line_slice = &line_bytes[..line_bytes.len() - 1]; // -1 because we know it ends in \n

                    if line_slice.is_empty() {
                        continue;
                    }

                    // Try parsing
                    let res = serde_json::from_slice::<PullModelStatus>(line_slice);
                    match res {
                        Ok(res) => yield res,
                        Err(e) => {
                            // If parsing fails, check if it's an internal API error
                            let err = serde_json::from_slice::<InternalOllamaError>(line_slice);
                            match err {
                                Ok(err) => Err(OllamaError::InternalError(err))?,
                                Err(_) => Err::<(), OllamaError>(e.into())?,
                            }
                        }
                    }
                }
            }
        };

        Ok(std::pin::Pin::from(Box::new(stream)))
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
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PullModelRequest {
    #[serde(rename = "name")]
    model_name: String,
    #[serde(rename = "insecure")]
    allow_insecure: bool,
    stream: bool,
}

/// A pull model status response from Ollama.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullModelStatus {
    #[serde(rename = "status")]
    pub message: String,
    pub digest: Option<String>,
    pub total: Option<u64>,
    pub completed: Option<u64>,
}
