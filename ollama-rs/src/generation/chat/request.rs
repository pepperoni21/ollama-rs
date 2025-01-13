use serde::Serialize;

use crate::generation::{
    completion::request::AbortSignal,
    options::GenerationOptions,
    parameters::FormatType,
    tools::{ToolGroup, ToolInfo},
};

use super::ChatMessage;

/// A chat message request to Ollama.
#[derive(Debug, Clone, Serialize)]
pub struct ChatMessageRequest {
    #[serde(rename = "model")]
    pub model_name: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ToolInfo>,
    pub options: Option<GenerationOptions>,
    pub template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<FormatType>,
    /// Must be false if tools are provided
    pub(crate) stream: bool,
    #[serde(skip)]
    pub abort_signal: Option<AbortSignal>,
    #[serde(skip)]
    pub(crate) timeout: Option<std::time::Duration>,
}

impl ChatMessageRequest {
    pub fn new(model_name: String, messages: Vec<ChatMessage>) -> Self {
        Self {
            model_name,
            messages,
            options: None,
            template: None,
            format: None,
            // Stream value will be overwritten by Ollama::send_chat_messages_stream() and Ollama::send_chat_messages() methods
            stream: false,
            tools: vec![],
            abort_signal: None,
            timeout: None,
        }
    }

    /// Additional model parameters listed in the documentation for the Modelfile
    pub fn options(mut self, options: GenerationOptions) -> Self {
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

    /// Tools that are available to the LLM.
    pub fn tools<T: ToolGroup>(mut self) -> Self {
        self.tools.clear();
        T::tool_info(&mut self.tools);

        self
    }

    /// Sets the abort signal for the request
    pub fn abort_signal(mut self, signal: AbortSignal) -> Self {
        self.abort_signal = Some(signal);
        self
    }

    /// Sets the timeout for the request
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}
