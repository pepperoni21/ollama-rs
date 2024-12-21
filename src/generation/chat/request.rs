use serde::Serialize;

use crate::generation::{options::GenerationOptions, parameters::FormatType};

use super::ChatMessage;

/// A chat message request to Ollama.
#[derive(Debug, Clone, Serialize)]
pub struct ChatMessageRequest {
    #[serde(rename = "model")]
    pub model_name: String,
    pub messages: Vec<ChatMessage>,
    pub options: Option<GenerationOptions>,
    pub template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<FormatType>,
    pub(crate) stream: bool,
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
}
