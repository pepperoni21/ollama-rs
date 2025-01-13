use std::sync::{atomic::AtomicBool, Arc};

use serde::Serialize;

use crate::generation::{
    images::Image,
    options::GenerationOptions,
    parameters::{FormatType, KeepAlive},
};

use super::GenerationContext;

#[derive(Debug, Clone)]
pub struct AbortSignal {
    pub(crate) abort_signal: Arc<AtomicBool>,
}

impl AbortSignal {
    pub fn new() -> Self {
        Self {
            abort_signal: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn abort(&self) {
        self.abort_signal
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn aborted(&self) -> bool {
        self.abort_signal.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// A generation request to Ollama.
#[derive(Debug, Clone, Serialize)]
pub struct GenerationRequest {
    #[serde(rename = "model")]
    pub model_name: String,
    pub prompt: String,
    pub suffix: Option<String>,
    pub images: Vec<Image>,
    pub options: Option<GenerationOptions>,
    pub system: Option<String>,
    pub template: Option<String>,
    pub context: Option<GenerationContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<FormatType>,
    pub keep_alive: Option<KeepAlive>,
    pub(crate) stream: bool,
    #[serde(skip)]
    pub abort_signal: Option<AbortSignal>,
}

impl GenerationRequest {
    pub fn new(model_name: String, prompt: String) -> Self {
        Self {
            model_name,
            prompt,
            suffix: None,
            images: Vec::new(),
            options: None,
            system: None,
            template: None,
            context: None,
            format: None,
            keep_alive: None,
            // Stream value will be overwritten by Ollama::generate_stream() and Ollama::generate() methods
            stream: false,
            abort_signal: None,
        }
    }

    /// Creates a new generation request with an suffix. Useful for code completion requests
    pub fn new_with_suffix(model_name: String, prompt: String, suffix: String) -> Self {
        let out = Self::new(model_name, prompt);
        out.suffix(suffix)
    }

    /// Adds a text after the model response
    pub fn suffix(mut self, suffix: String) -> Self {
        self.suffix = Some(suffix);
        self
    }

    /// A list of images to be used with the prompt
    pub fn images(mut self, images: Vec<Image>) -> Self {
        self.images = images;
        self
    }

    /// Add an image to be used with the prompt
    pub fn add_image(mut self, image: Image) -> Self {
        self.images.push(image);
        self
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

    /// Sets the abort signal for the request
    pub fn abort_signal(mut self, abort_signal: AbortSignal) -> Self {
        self.abort_signal = Some(abort_signal);
        self
    }
}
