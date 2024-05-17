pub mod prompts;
pub mod request;

pub use prompts::DEFAULT_SYSTEM_TEMPLATE;

use serde_json::{json, Value};
use std::collections::HashMap;
use serde_json::Map;
use std::sync::Arc;
use crate::generation::functions::Tool;

pub fn convert_to_openai_tool(tool: Arc<dyn Tool>) -> HashMap<String, Value> {
    let mut function = HashMap::new();
    function.insert("name".to_string(), Value::String(tool.name()));
    function.insert("description".to_string(), Value::String(tool.description()));
    function.insert("parameters".to_string(), tool.parameters());

    let mut result = HashMap::new();
    result.insert("type".to_string(), Value::String("function".to_string()));

    let mapp: Map<String, Value> = function.into_iter().collect();
    result.insert("function".to_string(), Value::Object(mapp));

    result
}
