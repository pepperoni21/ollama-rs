use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use crate::generation::chat::{ChatMessage, ChatMessageResponse};
use crate::generation::functions::pipelines::nous_hermes::DEFAULT_SYSTEM_TEMPLATE;
use crate::generation::functions::tools::Tool;
use crate::error::OllamaError;
use crate::generation::functions::pipelines::RequestParserBase;
use serde::{Deserialize, Serialize};
use regex::Regex;

pub fn convert_to_openai_tool(tool: Arc<dyn Tool>) -> Value {
    let mut function = HashMap::new();
    function.insert("name".to_string(), Value::String(tool.name()));
    function.insert("description".to_string(), Value::String(tool.description()));
    function.insert("parameters".to_string(), tool.parameters());

    let mut result = HashMap::new();
    result.insert("type".to_string(), Value::String("function".to_string()));

    let mapp: Map<String, Value> = function.into_iter().collect();
    result.insert("function".to_string(), Value::Object(mapp));

    json!(result)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NousFunctionCallSignature {
    pub name: String,
    pub arguments: Value,
}

pub struct NousFunctionCall {}

impl NousFunctionCall {
    pub async fn function_call_with_history(
        &self,
        model_name: String,
        tool_params: Value,
        tool: Arc<dyn Tool>,
    ) -> Result<ChatMessageResponse, OllamaError> {
        let result = tool.run(tool_params).await;
        match result {
            Ok(result) => Ok(ChatMessageResponse {
                model: model_name.clone(),
                created_at: "".to_string(),
                message: Some(ChatMessage::assistant(result)),
                done: true,
                final_data: None,
            }),
            Err(e) => Err(OllamaError::from(e)),
        }
    }

    pub fn format_tool_response(&self, function_response: &str) -> String {
        format!("<tool_call>\n{}\n</tool_call>\n", function_response)
    }

    pub fn extract_tool_response(&self, content: &str) -> Option<String> {
        let re = Regex::new(r"(?s)<tool_call>(.*?)</tool_call>").unwrap();
        re.captures(content).and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
    }
}

#[async_trait]
impl RequestParserBase for NousFunctionCall {
    async fn parse(
        &self,
        input: &str,
        model_name: String,
        tools: Vec<Arc<dyn Tool>>,
    ) -> Result<ChatMessageResponse, OllamaError> {
        //Extract between <tool_response> and </tool_response>
        let tool_response = self.extract_tool_response(input);
        match tool_response {
            Some(tool_response_str) => {
                let response_value: Result<NousFunctionCallSignature, serde_json::Error> = serde_json::from_str(&tool_response_str);
                match response_value {
                    Ok(response) => {
                        if let Some(tool) = tools.iter().find(|t| t.name() == response.name) {
                            let tool_params = response.arguments;
                            let result = self
                                .function_call_with_history(
                                    model_name.clone(),
                                    tool_params.clone(),
                                    tool.clone(),
                                )
                                .await?;
                            return Ok(result);
                        } else {
                            return Err(OllamaError::from("Tool not found".to_string()));
                        }
                    }
                    Err(e) => return Err(OllamaError::from(e)),
                }
            }
            None => return Err(OllamaError::from("Tool response not found".to_string())),
        }
    }

    async fn get_system_message(&self, tools: &[Arc<dyn Tool>]) -> ChatMessage {
        let tools_info: Vec<Value> = tools.iter().map(|tool| convert_to_openai_tool(tool.clone())).collect();
        let tools_json = serde_json::to_string(&tools_info).unwrap();
        let system_message_content = DEFAULT_SYSTEM_TEMPLATE.replace("{tools}", &tools_json);
        ChatMessage::system(system_message_content)
    }
}
