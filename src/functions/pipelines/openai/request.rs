use serde_json::Value;
use std::sync::Arc;
use crate::generation::chat::{ChatMessage, ChatMessageResponse};
use crate::generation::chat::request::{ChatMessageRequest};
use crate::generation::functions::pipelines::openai::DEFAULT_SYSTEM_TEMPLATE;
use crate::generation::functions::tools::Tool;
use crate::Ollama;
use crate::error::OllamaError;

#[derive(Clone)]
pub struct FunctionCallRequest {
    model_name: String,
    tools: Vec<Arc<dyn Tool>>,
}

impl FunctionCallRequest {
    pub fn new(model_name: &str, tools: Vec<Arc<dyn Tool>>) -> Self {
        FunctionCallRequest {
            model_name: model_name.to_string(),
            tools,
        }
    }

    pub async fn send(&self, ollama: &mut Ollama, input: &str) -> Result<ChatMessageResponse, OllamaError> {
        let system_message = self.get_system_message();
        ollama.send_chat_messages_with_history(
            ChatMessageRequest::new(self.model_name.clone(), vec![system_message.clone()]),
            "default".to_string(),
        ).await?;

        let user_message = ChatMessage::user(input.to_string());

        let result = ollama
            .send_chat_messages_with_history(
                ChatMessageRequest::new(self.model_name.clone(), vec![user_message]),
                "default".to_string(),
            ).await?;

        let response_content = result.message.clone().unwrap().content;
        let response_value: Value = match serde_json::from_str(&response_content) {
            Ok(value) => value,
            Err(e) => return Err(OllamaError::from(e.to_string())),
        };

        if let Some(function_call) = response_value.get("function_call") {
            if let Some(tool_name) = function_call.get("tool").and_then(Value::as_str) {
                if let Some(tool) = self.tools.iter().find(|t| t.name() == tool_name) {
                    let result = ollama.function_call_with_history(
                        ChatMessageRequest::new(self.model_name.clone(), vec![ChatMessage::user(tool_name.to_string())]),
                        tool.clone(),
                    ).await?;
                    return Ok(result);
                }
            }
        }

        Ok(result)
    }

    pub fn get_system_message(&self) -> ChatMessage {
        let tools_info: Vec<Value> = self.tools.iter().map(|tool| tool.parameters()).collect();
        let tools_json = serde_json::to_string(&tools_info).unwrap();
        let system_message_content = DEFAULT_SYSTEM_TEMPLATE.replace("{tools}", &tools_json);
        ChatMessage::system(system_message_content)
    }
}
