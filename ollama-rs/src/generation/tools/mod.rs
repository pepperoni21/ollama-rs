#[cfg_attr(docsrs, doc(cfg(feature = "tool-implementations")))]
#[cfg(feature = "tool-implementations")]
pub mod implementations;

use std::{future::Future, pin::Pin};

use schemars::{generate::SchemaSettings, JsonSchema, Schema};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// It's highly recommended that the `JsonSchema` has descriptions for all attributes.
/// Descriptions can be defined with `#[schemars(description = "Hi I am an attribute")]` above each attribute
// TODO enforce at compile-time
pub trait Tool: Send + Sync {
    type Params: Parameters;

    fn name() -> &'static str;
    fn description() -> &'static str;

    /// Call the tool.
    /// Note that returning an Err will cause it to be bubbled up. If you want the LLM to handle the error,
    /// return that error as a string.
    fn call(
        &mut self,
        parameters: Self::Params,
    ) -> impl Future<Output = Result<String>> + Send + Sync;
}

pub trait Parameters: DeserializeOwned + JsonSchema {}

impl<P: DeserializeOwned + JsonSchema> Parameters for P {}

pub(crate) trait ToolHolder: Send + Sync {
    fn call(
        &mut self,
        parameters: Value,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + '_ + Send + Sync>>;
}

impl<T: Tool> ToolHolder for T {
    fn call(
        &mut self,
        parameters: Value,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + '_ + Send + Sync>> {
        Box::pin(async move {
            // Json returned from the model can sometimes be in different formats, see https://github.com/pepperoni21/ollama-rs/issues/210
            // This is a work-around for this issue.
            let param_value = match serde_json::from_value(parameters.clone()) {
                // We first try with the ToolCallFunction format
                Ok(ToolCallFunction { name: _, arguments }) => arguments,
                Err(_err) => match serde_json::from_value::<ToolInfo>(parameters.clone()) {
                    Ok(ti) => ti.function.parameters.to_value(),
                    Err(_err) => parameters,
                },
            };

            let param = serde_json::from_value(param_value)?;

            T::call(self, param).await
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolInfo {
    #[serde(rename = "type")]
    pub tool_type: ToolType,
    pub function: ToolFunctionInfo,
}

impl ToolInfo {
    pub(crate) fn new<P: Parameters, T: Tool<Params = P>>() -> Self {
        let mut settings = SchemaSettings::draft07();
        settings.inline_subschemas = true;
        let generator = settings.into_generator();

        let parameters = generator.into_root_schema_for::<P>();

        Self {
            tool_type: ToolType::Function,
            function: ToolFunctionInfo {
                name: T::name().to_string(),
                description: T::description().to_string(),
                parameters,
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ToolType {
    #[serde(rename_all(deserialize = "PascalCase"))]
    Function,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolFunctionInfo {
    pub name: String,
    pub description: String,
    pub parameters: Schema,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub function: ToolCallFunction,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    // I don't love this (the Value)
    // But fixing it would be a big effort
    // FIXME
    #[serde(alias = "parameters")]
    pub arguments: Value,
}
