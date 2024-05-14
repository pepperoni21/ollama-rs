pub mod tools;
pub mod pipelines;

pub use tools::WeatherTool;
pub use tools::Scraper;
pub use tools::DDGSearcher;

use async_trait::async_trait;
use serde_json::{Value, json};
use std::error::Error;
use crate::generation::chat::ChatMessage;


pub trait FunctionCallBase: Send + Sync {
    fn name(&self) -> String;
}

#[async_trait]
pub trait FunctionCall: FunctionCallBase {
    async fn call(&self, params: Value) -> Result<Value, Box<dyn Error>>;
}

pub struct DefaultFunctionCall {}

impl FunctionCallBase for DefaultFunctionCall {
    fn name(&self) -> String {
        "default_function".to_string()
    }
}


pub fn convert_to_ollama_tool(tool: &dyn crate::generation::functions::tools::Tool) -> Value {
    let schema = tool.parameters();
    json!({
        "name": tool.name(),
        "properties": schema["properties"],
        "required": schema["required"]
    })
}


pub fn parse_response(message: &ChatMessage) -> Result<String, String> {
    let content = &message.content;
    let value: Value = serde_json::from_str(content).map_err(|e| e.to_string())?;

    if let Some(function_call) = value.get("function_call") {
        if let Some(arguments) = function_call.get("arguments") {
            return Ok(arguments.to_string());
        }
        return Err("`arguments` missing from `function_call`".to_string());
    }
    Err("`function_call` missing from `content`".to_string())
}
