use serde::{Deserialize, Serialize};

use crate::generation::options::GenerationOptions;

use super::GenerationContext;

#[derive(Debug, Clone, Serialize)]
pub struct GenerationRequest {
    #[serde(rename = "model")]
    pub model_name: String,
    pub prompt: String,
    pub options: Option<GenerationOptions>,
    pub system: Option<String>,
    pub template: Option<String>,
    pub context: Option<GenerationContext>,
    pub format: Option<FormatEnum>,
    pub(crate) stream: bool,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FormatEnum {
    Json,
}
impl GenerationRequest {
    pub fn new(model_name: String, prompt: String) -> Self {
        Self {
            model_name,
            prompt,
            options: None,
            system: None,
            template: None,
            context: None,
            // The format to return a response in. Currently the only accepted value is `json`
            format: None,
            // Stream value will be overwritten by Ollama::generate_stream() and Ollama::generate() methods
            stream: false,
        }
    }

    /// Additional model parameters listed in the documentation for the Modelfile
    pub fn options(mut self, options: GenerationOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// System prompt to (overrides what is defined in the Modelfile)
    pub fn system(mut self, system: String) -> Self {
        self.system = Some(system);
        self
    }

    /// The full prompt or prompt template (overrides what is defined in the Modelfile)
    pub fn template(mut self, template: String) -> Self {
        self.template = Some(template);
        self
    }

    /// The context parameter returned from a previous request to /generate, this can be used to keep a short conversational memory
    pub fn context(mut self, context: GenerationContext) -> Self {
        self.context = Some(context);
        self
    }
}
