use serde::{Deserialize, Serialize};

use crate::{
    generation::{
        parameters::{FormatType, KeepAlive, ThinkType},
        tools::{Tool, ToolInfo},
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
    /// Set by the chat send methods before the request is sent.
    #[serde(default)]
    pub(crate) stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub think: Option<ThinkType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,
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
            logprobs: None,
            top_logprobs: None,
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

    /// Add a tool definition that is available to the LLM.
    ///
    /// This consumes the tool value only to identify its schema. The request does
    /// not retain or execute the tool. When using `Ollama::send_chat_messages_stream`,
    /// callers are responsible for consuming streamed tool calls and appending
    /// tool results to the next request.
    pub fn add_tool<T: Tool>(mut self, _tool: T) -> Self {
        self.tools.push(ToolInfo::from_tool::<T>());
        self
    }

    /// Used to control whether thinking/reasoning models will think before responding
    pub fn think(mut self, think: impl Into<ThinkType>) -> Self {
        self.think = Some(think.into());
        self
    }

    /// Used to control whether to return log probabilities for each token
    pub fn logprobs(mut self, logprobs: bool) -> Self {
        self.logprobs = Some(logprobs);
        self
    }

    /// Used to control the number of top log probabilities to return for each token
    pub fn top_logprobs(mut self, top_logprobs: u32) -> Self {
        self.top_logprobs = Some(top_logprobs);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::parameters::{FormatType, JsonSchema, JsonStructure};
    use serde::Deserialize;
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

    #[allow(dead_code)]
    #[derive(Debug, Deserialize, JsonSchema)]
    struct TestToolParams {
        city: String,
    }

    struct TestTool;

    impl Tool for TestTool {
        type Params = TestToolParams;

        fn name() -> &'static str {
            "test_weather"
        }

        fn description() -> &'static str {
            "Gets test weather"
        }

        async fn call(
            &mut self,
            _parameters: Self::Params,
        ) -> crate::generation::tools::Result<String> {
            Ok("sunny".to_string())
        }
    }

    #[test]
    fn add_tool_serializes_tool_schema_for_streaming_request() {
        let mut request = ChatMessageRequest::new(
            "model".to_string(),
            vec![ChatMessage::user("hello".to_string())],
        )
        .add_tool(TestTool);

        request.stream = true;

        let value = serde_json::to_value(&request).expect("serialize request");

        assert_eq!(
            value.get("stream").expect("stream field present"),
            &Value::Bool(true)
        );

        let tools = value
            .get("tools")
            .expect("tools field present")
            .as_array()
            .expect("tools serialized as array");

        assert_eq!(tools.len(), 1);

        let function = tools[0]
            .get("function")
            .expect("function field present")
            .as_object()
            .expect("function serialized as object");

        assert_eq!(
            function.get("name").expect("name present"),
            &Value::String("test_weather".to_string())
        );
        assert_eq!(
            function.get("description").expect("description present"),
            &Value::String("Gets test weather".to_string())
        );
        assert!(function.contains_key("parameters"));
    }
}
