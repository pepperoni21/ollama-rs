pub mod prompts;
pub mod parsers;
pub mod request;

pub use prompts::{DEFAULT_SYSTEM_TEMPLATE ,DEFAULT_RESPONSE_FUNCTION};
pub use request::FunctionCallRequest;
pub use parsers::{generate_system_message, parse_response};

use std::sync::Arc;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::error::Error;
use crate::generation::functions::{FunctionCall, FunctionCallBase};
use crate::generation::chat::{ChatMessage, ChatMessageResponse};
use crate::generation::chat::request::{ChatMessageRequest};
use crate::generation::functions::tools::Tool;
use crate::error::OllamaError;


pub struct OpenAIFunctionCall {
    pub name: String,
}

impl OpenAIFunctionCall {
    pub fn new(name: &str) -> Self {
        OpenAIFunctionCall {
            name: name.to_string(),
        }
    }
}

impl FunctionCallBase for OpenAIFunctionCall {
    fn name(&self) -> String {
        "openai".to_string()
    }
}

#[async_trait]
impl FunctionCall for OpenAIFunctionCall {
    async fn call(&self, params: Value) -> Result<Value, Box<dyn Error>> {
        // Simulate a function call by returning a simple JSON value
        Ok(json!({ "result": format!("Function {} called with params: {}", self.name, params) }))
    }
}


impl crate::Ollama {
    pub async fn function_call_with_history(
        &self,
        request: ChatMessageRequest,
        tool: Arc<dyn Tool>,
    ) -> Result<ChatMessageResponse, OllamaError> {
        let function_call = OpenAIFunctionCall::new(&tool.name());
        let params = tool.parameters();
        let result = function_call.call(params).await?;
        Ok(ChatMessageResponse {
            model: request.model_name,
            created_at: "".to_string(),
            message: Some(ChatMessage::assistant(result.to_string())),
            done: true,
            final_data: None,
        })
    }

    pub async fn function_call(
        &self,
        request: ChatMessageRequest,
    ) -> crate::error::Result<ChatMessageResponse> {
        let mut request = request;
        request.stream = false;

        let url = format!("{}api/chat", self.url_str());
        let serialized = serde_json::to_string(&request).map_err(|e| e.to_string())?;
        let res = self
            .reqwest_client
            .post(url)
            .body(serialized)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(res.text().await.unwrap_or_else(|e| e.to_string()).into());
        }

        let bytes = res.bytes().await.map_err(|e| e.to_string())?;
        let res =
            serde_json::from_slice::<ChatMessageResponse>(&bytes).map_err(|e| e.to_string())?;

        Ok(res)
    }
}