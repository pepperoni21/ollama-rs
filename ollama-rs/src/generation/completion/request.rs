use std::borrow::Cow;

use serde::Serialize;

use crate::{
    generation::{
        images::Image,
        parameters::{FormatType, KeepAlive},
    },
    models::ModelOptions,
};

use super::GenerationContext;

/// A generation request to Ollama.
#[derive(Debug, Clone, Serialize)]
pub struct GenerationRequest<'a> {
    #[serde(rename = "model")]
    pub model_name: String,
    pub prompt: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<Cow<'a, str>>,
    pub images: Vec<Image>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ModelOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<GenerationContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<FormatType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<KeepAlive>,
    pub think: Option<bool>,
}

impl<'a> GenerationRequest<'a> {
    pub fn new(model_name: String, prompt: impl Into<Cow<'a, str>>) -> Self {
        Self {
            model_name,
            prompt: prompt.into(),
            suffix: None,
            images: Vec::new(),
            options: None,
            system: None,
            template: None,
            raw: None,
            context: None,
            format: None,
            keep_alive: None,
            think: None,
        }
    }

    /// Creates a new generation request with an suffix. Useful for code completion requests
    pub fn new_with_suffix(model_name: String, prompt: String, suffix: String) -> Self {
        let out = Self::new(model_name, prompt);
        out.suffix(suffix)
    }

    /// Adds a text after the model response
    pub fn suffix(mut self, suffix: impl Into<Cow<'a, str>>) -> Self {
        self.suffix = Some(suffix.into());
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
    pub fn options(mut self, options: ModelOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// System prompt to (overrides what is defined in the Modelfile)
    pub fn system(mut self, system: impl Into<Cow<'a, str>>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// The full prompt or prompt template (overrides what is defined in the Modelfile)
    pub fn template(mut self, template: impl Into<Cow<'a, str>>) -> Self {
        self.template = Some(template.into());
        self
    }

    /// If `true` no formatting will be applied to the prompt. You may choose to use the `raw` parameter if you are specifying a full templated prompt in your request to the API
    pub fn raw(mut self, raw: bool) -> Self {
        self.raw = Some(raw);
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

    /// Used to control whether thinking/reasoning models will think before responding
    pub fn think(mut self, think: bool) -> Self {
        self.think = Some(think);
        self
    }
}
