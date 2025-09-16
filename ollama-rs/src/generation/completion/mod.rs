use serde::{Deserialize, Serialize};

use crate::{error::OllamaError, Ollama};

use request::GenerationRequest;

pub mod request;

#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
/// A stream of `GenerationResponse` objects
pub type GenerationResponseStream =
    futures_util::stream::BoxStream<'static, crate::error::Result<GenerationResponse>>;

#[derive(Serialize)]
struct WithStreamField<T> {
    stream: bool,
    #[serde(flatten)]
    rest: T,
}

impl Ollama {
    #[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
    #[cfg(feature = "stream")]
    /// Completion generation with streaming.
    /// Returns a stream of `GenerationResponse` objects
    pub async fn generate_stream(
        &self,
        request: GenerationRequest<'_>,
    ) -> crate::error::Result<GenerationResponseStream> {
        crate::stream::map_response(self.send_generation_request(request, true).await?).await
    }

    /// Completion generation with a single response.
    /// Returns a single `GenerationResponse` object
    pub async fn generate(
        &self,
        request: GenerationRequest<'_>,
    ) -> crate::error::Result<GenerationResponse> {
        crate::map_response(self.send_generation_request(request, false).await?).await
    }

    async fn send_generation_request(
        &self,
        request: GenerationRequest<'_>,
        stream: bool,
    ) -> Result<reqwest::Response, OllamaError> {
        let url = format!("{}api/generate", self.url_str());
        let builder = self.reqwest_client.post(url);
        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());
        let res = builder
            .json(&WithStreamField {
                stream,
                rest: request,
            })
            .send()
            .await?;
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
    /// Time spent in nanoseconds loading the model
    pub load_duration: Option<u64>,
    /// Number of tokens in the prompt
    pub prompt_eval_count: Option<u64>,
    /// Time spent in nanoseconds evaluating the prompt
    pub prompt_eval_duration: Option<u64>,
    /// Number of tokens in the response
    pub eval_count: Option<u64>,
    /// Time spent in nanoseconds generating the response
    pub eval_duration: Option<u64>,
    /// Contains the text that was inside thinking tags in the original model output when ChatMessageRequest.Think is enabled.
    pub thinking: Option<String>,
}
