use crate::generation::chat::ChatMessage;
use serde_json::Value;
use crate::generation::functions::pipelines::openai::DEFAULT_SYSTEM_TEMPLATE;
use crate::generation::functions::tools::Tool;

pub fn parse_response(message: &ChatMessage) -> Result<Value, String> {
    let content = &message.content;
    let value: Value = serde_json::from_str(content).map_err(|e| e.to_string())?;

    if let Some(function_call) = value.get("function_call") {
        Ok(function_call.clone())
    } else {
        Ok(value)
    }
}

pub fn generate_system_message(tools: &[&dyn Tool]) -> ChatMessage {
    let tools_info: Vec<Value> = tools.iter().map(|tool| tool.parameters()).collect();
    let tools_json = serde_json::to_string(&tools_info).unwrap();
    let system_message_content = DEFAULT_SYSTEM_TEMPLATE.replace("{tools}", &tools_json);
    ChatMessage::system(system_message_content)
}

