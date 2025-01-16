use serde::{Deserialize, Serialize};

use crate::{error::OllamaError, Ollama};

use request::GenerationRequest;

pub mod request;

#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
/// A stream of `GenerationResponse` objects
pub type GenerationResponseStream = std::pin::Pin<
    Box<
        dyn tokio_stream::Stream<Item = crate::error::Result<GenerationResponseStreamChunk>> + Send,
    >,
>;
pub type GenerationResponseStreamChunk = Vec<GenerationResponse>;

impl Ollama {
    #[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
    #[cfg(feature = "stream")]
    /// Completion generation with streaming.
    /// Returns a stream of `GenerationResponse` objects
    pub async fn generate_stream(
        &self,
        request: GenerationRequest,
    ) -> crate::error::Result<GenerationResponseStream> {
        use tokio_stream::StreamExt;

        use crate::error::OllamaError;

        let mut request = request;
        request.stream = true;

        let url = format!("{}api/generate", self.url_str());
        let serialized = serde_json::to_string(&request)?;
        let mut builder = self.reqwest_client.post(url);

        if let Some(timeout) = request.timeout {
            builder = builder.timeout(timeout);
        }

        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());

        let res = builder.body(serialized).send().await?;

        if !res.status().is_success() {
            return Err(OllamaError::Other(
                res.text().await.unwrap_or_else(|e| e.to_string()),
            ));
        }

        let stream = Box::new(res.bytes_stream().map(move |res| {
            if let Some(abort_signal) = request.abort_signal.as_ref() {
                if abort_signal.aborted() {
                    return Err(OllamaError::Abort);
                }
            }
            match res {
                Ok(bytes) => {
                    let res = serde_json::Deserializer::from_slice(&bytes).into_iter();
                    let res = res
                        .filter_map(Result::ok) // Filter out the errors
                        .collect::<Vec<GenerationResponse>>();
                    Ok(res)
                }
                Err(e) => Err(OllamaError::Other(format!(
                    "Failed to read response: {}",
                    e
                ))),
            }
        }));

        Ok(std::pin::Pin::from(stream))
    }

    /// Completion generation with a single response.
    /// Returns a single `GenerationResponse` object
    pub async fn generate(
        &self,
        request: GenerationRequest,
    ) -> crate::error::Result<GenerationResponse> {
        let mut request = request;
        request.stream = false;

        let url = format!("{}api/generate", self.url_str());
        let serialized = serde_json::to_string(&request)?;
        let builder = self.reqwest_client.post(url);

        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());

        let mut builder = builder.body(serialized);
        if let Some(timeout) = request.timeout {
            builder = builder.timeout(timeout);
        }
        let res = builder.send().await?;

        if !res.status().is_success() {
            return Err(OllamaError::Other(
                res.text().await.unwrap_or_else(|e| e.to_string()),
            ));
        }

        let res = res.bytes().await?;
        let res = serde_json::from_slice::<GenerationResponse>(&res)?;

        Ok(res)
    }
}

/// An encoding of a conversation returned by Ollama after a completion request, this can be sent in a new request to keep a conversational memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationContext(pub Vec<i32>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResponse {
    /// The name of the model used for the completion.
    pub model: String,
    /// The creation time of the completion, in such format: `2023-08-04T08:52:19.385406455-07:00`.
    pub created_at: String,
    /// The response of the completion. This can be the entire completion or only a token if the completion is streaming.
    pub response: String,
    /// Whether the completion is done. If the completion is streaming, this will be false until the last response.
    pub done: bool,
    /// An encoding of the conversation used in this response, this can be sent in the next request to keep a conversational memory
    pub context: Option<GenerationContext>,
    /// Time spent generating the response
    pub total_duration: Option<u64>,
    /// Number of tokens in the prompt
    pub prompt_eval_count: Option<u16>,
    /// Time spent in nanoseconds evaluating the prompt
    pub prompt_eval_duration: Option<u64>,
    /// Number of tokens in the response
    pub eval_count: Option<u16>,
    /// Time spent in nanoseconds generating the response
    pub eval_duration: Option<u64>,
}
