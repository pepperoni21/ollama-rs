use crate::error::OllamaError;
use crate::generation::chat::{ChatMessage, ChatMessageResponse};
use crate::generation::functions::pipelines::meta_llama::DEFAULT_SYSTEM_TEMPLATE;
use crate::generation::functions::pipelines::RequestParserBase;
use crate::generation::functions::tools::Tool;
use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

pub fn convert_to_llama_tool(tool: &Arc<dyn Tool>) -> Value {
    let mut function = HashMap::new();
    function.insert("name".to_string(), Value::String(tool.name()));
    function.insert("description".to_string(), Value::String(tool.description()));
    function.insert("parameters".to_string(), tool.parameters());
    json!(format!(
        "Use the function '{name}' to '{description}': {json}",
        name = tool.name(),
        description = tool.description(),
        json = json!(function)
    ))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LlamaFunctionCallSignature {
    pub function: String, //name of the tool
    pub arguments: Value,
}

pub struct LlamaFunctionCall {}

impl LlamaFunctionCall {
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
                message: Some(ChatMessage::assistant(result.to_string())),
                done: true,
                final_data: None,
            }),
            Err(e) => Err(OllamaError::from(e)),
        }
    }

    fn clean_tool_call(&self, json_str: &str) -> String {
        json_str
            .trim()
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim()
            .to_string()
            .replace("{{", "{")
            .replace("}}", "}")
            .replace("\\\"", "\"")
    }

    fn parse_tool_response(&self, response: &str) -> Vec<LlamaFunctionCallSignature> {
        let function_regex = Regex::new(r"<function=(\w+)>(.*?)</function>").unwrap();
        println!("Response: {}", response);

        function_regex
            .captures_iter(response)
            .filter_map(|caps| {
                let function_name = caps.get(1)?.as_str().to_string();
                let args_string = caps.get(2)?.as_str();

                match serde_json::from_str(args_string) {
                    Ok(arguments) => Some(LlamaFunctionCallSignature {
                        function: function_name,
                        arguments,
                    }),
                    Err(error) => {
                        println!("Error parsing function arguments: {}", error);
                        None
                    }
                }
            })
            .collect()
    }
}

#[async_trait]
impl RequestParserBase for LlamaFunctionCall {
    async fn parse(
        &self,
        input: &str,
        model_name: String,
        tools: Vec<Arc<dyn Tool>>,
    ) -> Result<ChatMessageResponse, OllamaError> {
        let function_calls = self.parse_tool_response(&self.clean_tool_call(input));

        if function_calls.is_empty() {
            return Err(OllamaError::from(
                "No valid function calls found".to_string(),
            ));
        }

        let mut results = Vec::new();

        for call in function_calls {
            if let Some(tool) = tools.iter().find(|t| t.name() == call.function) {
                let tool_params = call.arguments;
                let result = self
                    .function_call_with_history(model_name.clone(), tool_params, tool.clone())
                    .await?;
                results.push(result);
            } else {
                return Err(OllamaError::from(format!(
                    "Tool '{}' not found",
                    call.function
                )));
            }
        }

        let combined_message = results
            .into_iter()
            .map(|r| r.message.map_or_else(String::new, |m| m.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(ChatMessageResponse {
            model: model_name,
            created_at: "".to_string(),
            message: Some(ChatMessage::assistant(combined_message)),
            done: true,
            final_data: None,
        })
    }

    async fn get_system_message(&self, tools: &[Arc<dyn Tool>]) -> ChatMessage {
        let tools_info: Vec<Value> = tools.iter().map(convert_to_llama_tool).collect();
        let tools_json = serde_json::to_string(&tools_info).unwrap();
        let system_message_content = DEFAULT_SYSTEM_TEMPLATE.replace("{tools}", &tools_json);
        ChatMessage::system(system_message_content)
    }
}
