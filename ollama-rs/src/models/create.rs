use serde::{Deserialize, Serialize};

use crate::{error::OllamaError, generation::chat::ChatMessage, Ollama};

use super::ModelOptions;

/// A stream of `CreateModelStatus` objects
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
pub type CreateModelStatusStream =
    futures_util::stream::BoxStream<'static, crate::error::Result<CreateModelStatus>>;

impl Ollama {
    #[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
    #[cfg(feature = "stream")]
    /// Create a model with streaming, meaning that each new status will be streamed.
    pub async fn create_model_stream(
        &self,
        mut request: CreateModelRequest,
    ) -> crate::error::Result<CreateModelStatusStream> {
        request.stream = true;
        let res = self.send_model_request(request).await?;
        crate::stream::map_response(res).await
    }

    /// Create a model with a single response, only the final status will be returned.
    pub async fn create_model(
        &self,
        request: CreateModelRequest,
    ) -> crate::error::Result<CreateModelStatus> {
        let res = self.send_model_request(request).await?;
        if res.status().is_success() {
            let res = res.bytes().await?;
            Ok(serde_json::from_slice::<CreateModelStatus>(&res)?)
        } else {
            Err(OllamaError::Other(res.text().await?))
        }
    }
    async fn send_model_request(
        &self,
        request: CreateModelRequest,
    ) -> reqwest::Result<reqwest::Response> {
        let url = format!("{}api/create", self.url_str());
        let builder = self.reqwest_client.post(url);

        #[cfg(feature = "headers")]
        let builder = builder.headers(self.request_headers.clone());

        builder.json(&request).send().await
    }
}

#[derive(Serialize)]
pub enum QuantizationType {
    #[serde(rename = "q2_K")]
    Q2K,
    #[serde(rename = "q3_K_L")]
    Q3KL,
    #[serde(rename = "q3_K_M")]
    Q3KM,
    #[serde(rename = "q3_K_S")]
    Q3KS,
    #[serde(rename = "q4_0")]
    Q40,
    #[serde(rename = "q4_1")]
    Q41,
    #[serde(rename = "q4_K_M")]
    Q4KM,
    #[serde(rename = "q4_K_S")]
    Q4KS,
    #[serde(rename = "q5_0")]
    Q50,
    #[serde(rename = "q5_1")]
    Q51,
    #[serde(rename = "q5_K_M")]
    Q5KM,
    #[serde(rename = "q5_K_S")]
    Q5KS,
    #[serde(rename = "q6_K")]
    Q6K,
    #[serde(rename = "q8_0")]
    Q80,
}

/// A create model request to Ollama.
#[derive(Serialize)]
pub struct CreateModelRequest {
    /// Name of the model to create
    #[serde(rename = "model")]
    model_name: String,
    /// Name of an existing model to create the new model from
    #[serde(rename = "from")]
    from_model: Option<String>,
    /// A dictionary of file names to SHA256 digests of blobs to create the model from
    #[serde(skip_serializing_if = "Option::is_none")]
    files: Option<std::collections::HashMap<String, String>>,
    /// A dictionary of file names to SHA256 digests of blobs for LORA adapters
    #[serde(skip_serializing_if = "Option::is_none")]
    adapters: Option<std::collections::HashMap<String, String>>,
    /// The prompt template for the model
    #[serde(skip_serializing_if = "Option::is_none")]
    template: Option<String>,
    /// A string or list of strings containing the license or licenses for the model
    #[serde(skip_serializing_if = "Option::is_none")]
    license: Option<Vec<String>>,
    /// A string containing the system prompt for the model
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    /// A dictionary of parameters for the model
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<ModelOptions>,
    /// A list of message objects used to create a conversation
    #[serde(skip_serializing_if = "Option::is_none")]
    messages: Option<Vec<ChatMessage>>,
    stream: bool,
    /// Quantize a non-quantized model
    #[serde(skip_serializing_if = "Option::is_none")]
    quantize: Option<QuantizationType>,
}

impl CreateModelRequest {
    pub fn new(model_name: String) -> Self {
        Self {
            model_name,
            from_model: None,
            files: None,
            adapters: None,
            template: None,
            license: None,
            system: None,
            parameters: None,
            messages: None,
            stream: false,
            quantize: None,
        }
    }

    pub fn from_model(mut self, from_model: String) -> Self {
        self.from_model = Some(from_model);
        self
    }

    pub fn files(mut self, files: std::collections::HashMap<String, String>) -> Self {
        self.files = Some(files);
        self
    }

    pub fn adapters(mut self, adapters: std::collections::HashMap<String, String>) -> Self {
        self.adapters = Some(adapters);
        self
    }

    pub fn template(mut self, template: String) -> Self {
        self.template = Some(template);
        self
    }

    pub fn license(mut self, license: String) -> Self {
        self.license = Some(vec![license]);
        self
    }

    pub fn licenses(mut self, licenses: Vec<String>) -> Self {
        self.license = Some(licenses);
        self
    }

    pub fn system(mut self, system: String) -> Self {
        self.system = Some(system);
        self
    }

    pub fn parameters(mut self, parameters: ModelOptions) -> Self {
        self.parameters = Some(parameters);
        self
    }

    pub fn messages(mut self, messages: Vec<ChatMessage>) -> Self {
        self.messages = Some(messages);
        self
    }

    pub fn quantize(mut self, quantize: QuantizationType) -> Self {
        self.quantize = Some(quantize);
        self
    }
}

/// A create model status response from Ollama.
#[derive(Deserialize, Debug)]
pub struct CreateModelStatus {
    #[serde(rename = "status")]
    pub message: String,
}
