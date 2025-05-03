/// Modules related to model operations.
///
/// These modules provide functionality for copying, creating, deleting,
/// listing, pulling, pushing, and showing information about models.
pub mod copy;
pub mod create;
pub mod delete;
pub mod list_local;
pub mod pull;
pub mod push;
pub mod show_info;

#[cfg(feature = "modelfile")]
use modelfile::modelfile::Modelfile;

#[cfg(feature = "modelfile")]
use serde_with;

use serde::{Deserialize, Serialize};

/// Represents a local model pulled from Ollama.
///
/// This struct contains information about a model that has been pulled
/// from the Ollama service, including its name, modification date, and size.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModel {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
}

/// Represents information about a model.
///
/// This struct contains various fields that describe a model's attributes,
/// such as its license, file, parameters, and template.
/// Some fields may be empty if the model does not have them.
///
/// By default the modelfile is a string, but if the `modelfile` feature is enabled,
/// it will be a `Modelfile` struct. See the modelfile crate for more information.
#[cfg_attr(feature = "modelfile", serde_with::serde_as)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    #[serde(default = "String::new")]
    pub license: String,
    #[cfg(feature = "modelfile")]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub modelfile: Modelfile,
    #[cfg(not(feature = "modelfile"))]
    #[serde(default = "String::new")]
    pub modelfile: String,
    #[serde(default = "String::new")]
    pub parameters: String,
    #[serde(default = "String::new")]
    pub template: String,
    #[serde(default = "serde_json::Map::new")]
    pub model_info: serde_json::Map<String, serde_json::Value>,
    #[serde(default = "Vec::new")]
    pub capabilities: Vec<String>,
}

// Options for generation requests to Ollama.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) mirostat: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) mirostat_eta: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) mirostat_tau: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) num_ctx: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) num_gqa: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) num_gpu: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) num_thread: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) repeat_last_n: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) repeat_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) seed: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) tfs_z: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) num_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) top_p: Option<f32>,
}

impl ModelOptions {
    /// Enable Mirostat sampling for controlling perplexity. (default: 0, 0 = disabled, 1 = Mirostat, 2 = Mirostat 2.0)
    pub fn mirostat(mut self, mirostat: u8) -> Self {
        self.mirostat = Some(mirostat);
        self
    }

    /// Influences how quickly the algorithm responds to feedback from the generated text. A lower learning rate will result in slower adjustments, while a higher learning rate will make the algorithm more responsive. (Default: 0.1)
    pub fn mirostat_eta(mut self, mirostat_eta: f32) -> Self {
        self.mirostat_eta = Some(mirostat_eta);
        self
    }

    /// Controls the balance between coherence and diversity of the output. A lower value will result in more focused and coherent text. (Default: 5.0)
    pub fn mirostat_tau(mut self, mirostat_tau: f32) -> Self {
        self.mirostat_tau = Some(mirostat_tau);
        self
    }

    /// Sets the size of the context window used to generate the next token. (Default: 2048)
    pub fn num_ctx(mut self, num_ctx: u64) -> Self {
        self.num_ctx = Some(num_ctx);
        self
    }

    /// The number of GQA groups in the transformer layer. Required for some models, for example it is 8 for llama2:70b
    pub fn num_gqa(mut self, num_gqa: u32) -> Self {
        self.num_gqa = Some(num_gqa);
        self
    }

    /// The number of layers to send to the GPU(s). On macOS it defaults to 1 to enable metal support, 0 to disable.
    pub fn num_gpu(mut self, num_gpu: u32) -> Self {
        self.num_gpu = Some(num_gpu);
        self
    }

    /// Sets the number of threads to use during computation. By default, Ollama will detect this for optimal performance. It is recommended to set this value to the number of physical CPU cores your system has (as opposed to the logical number of cores).
    pub fn num_thread(mut self, num_thread: u32) -> Self {
        self.num_thread = Some(num_thread);
        self
    }

    /// Sets how far back for the model to look back to prevent repetition. (Default: 64, 0 = disabled, -1 = num_ctx)
    pub fn repeat_last_n(mut self, repeat_last_n: i32) -> Self {
        self.repeat_last_n = Some(repeat_last_n);
        self
    }

    /// Sets how strongly to penalize repetitions. A higher value (e.g., 1.5) will penalize repetitions more strongly, while a lower value (e.g., 0.9) will be more lenient. (Default: 1.1)
    pub fn repeat_penalty(mut self, repeat_penalty: f32) -> Self {
        self.repeat_penalty = Some(repeat_penalty);
        self
    }

    /// The temperature of the model. Increasing the temperature will make the model answer more creatively. (Default: 0.8)
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sets the random number seed to use for generation. Setting this to a specific number will make the model generate the same text for the same prompt. (Default: 0)
    pub fn seed(mut self, seed: i32) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Sets the stop sequences to use. When this pattern is encountered the LLM will stop generating text and return. Multiple stop patterns may be set by specifying multiple separate `stop` parameters in a modelfile.
    pub fn stop(mut self, stop: Vec<String>) -> Self {
        self.stop = Some(stop);
        self
    }

    /// Tail free sampling is used to reduce the impact of less probable tokens from the output. A higher value (e.g., 2.0) will reduce the impact more, while a value of 1.0 disables this setting. (default: 1)
    pub fn tfs_z(mut self, tfs_z: f32) -> Self {
        self.tfs_z = Some(tfs_z);
        self
    }

    /// Maximum number of tokens to predict when generating text. (Default: 128, -1 = infinite generation, -2 = fill context)
    pub fn num_predict(mut self, num_predict: i32) -> Self {
        self.num_predict = Some(num_predict);
        self
    }

    /// Reduces the probability of generating nonsense. A higher value (e.g. 100) will give more diverse answers, while a lower value (e.g. 10) will be more conservative. (Default: 40)
    pub fn top_k(mut self, top_k: u32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// Works together with top-k. A higher value (e.g., 0.95) will lead to more diverse text, while a lower value (e.g., 0.5) will generate more focused and conservative text. (Default: 0.9)
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }
}
