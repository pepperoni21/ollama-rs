use serde::{Deserialize, Serialize};

use crate::{
    generation::{
        parameters::{FormatType, KeepAlive, ThinkType},
        tools::ToolInfo,
    },
    models::ModelOptions,
};

use super::ChatMessage;

/// A chat message request to Ollama.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageRequest {
    #[serde(rename = "model")]
    pub model_name: String,
    pub messages: Vec<ChatMessage>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ToolInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ModelOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<FormatType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<KeepAlive>,
    /// Must be false if tools are provided
    #[serde(default)]
    pub(crate) stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub think: Option<ThinkType>,
}

impl ChatMessageRequest {
    pub fn new(model_name: String, messages: Vec<ChatMessage>) -> Self {
        Self {
            model_name,
            messages,
            options: None,
            template: None,
            format: None,
            keep_alive: None,
            // Stream value will be overwritten by Ollama::send_chat_messages_stream() and Ollama::send_chat_messages() methods
            stream: false,
            tools: vec![],
            think: None,
        }
    }

    /// Additional model parameters listed in the documentation for the Modelfile
    pub fn options(mut self, options: ModelOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// The full prompt or prompt template (overrides what is defined in the Modelfile)
    pub fn template(mut self, template: String) -> Self {
        self.template = Some(template);
        self
    }

    /// The format to return a response in.
    pub fn format(mut self, format: FormatType) -> Self {
        self.format = Some(format);
        self
    }

    /// Used to control how long a model stays loaded in memory, by default models are unloaded after 5 minutes of inactivity
    pub fn keep_alive(mut self, keep_alive: KeepAlive) -> Self {
        self.keep_alive = Some(keep_alive);
        self
    }

    /// Tools that are available to the LLM.
    pub fn tools(mut self, tools: Vec<ToolInfo>) -> Self {
        self.tools = tools;
        self
    }

    /// Used to control whether thinking/reasoning models will think before responding
    pub fn think(mut self, think: impl Into<ThinkType>) -> Self {
        self.think = Some(think.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::parameters::{FormatType, JsonSchema, JsonStructure};
    use serde_json::Value;

    #[allow(dead_code)]
    #[derive(Debug, JsonSchema)]
    struct StructuredReply {
        answer: String,
    }

    #[test]
    fn serializes_structured_json_format() {
        let request = ChatMessageRequest::new(
            "model".to_string(),
            vec![ChatMessage::user("hello".to_string())],
        )
        .format(FormatType::StructuredJson(Box::new(JsonStructure::new::<
            StructuredReply,
        >())));

        let value = serde_json::to_value(&request).expect("serialize request");

        let format = value
            .get("format")
            .expect("format field present")
            .as_object()
            .expect("format serialized as object");

        assert_eq!(
            format
                .get("type")
                .expect("schema type present")
                .as_str()
                .expect("type is string"),
            "object"
        );

        let properties = format
            .get("properties")
            .expect("schema has properties")
            .as_object()
            .expect("properties serialized as object");

        assert!(properties.contains_key("answer"));

        assert_eq!(
            value.get("stream").expect("stream field present"),
            &Value::Bool(false)
        );
    }
}
