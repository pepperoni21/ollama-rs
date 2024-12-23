#[cfg_attr(docsrs, doc(cfg(feature = "tool-implementations")))]
#[cfg(feature = "tool-implementations")]
pub mod implementations;

use std::{error::Error, future::Future};

use schemars::{gen::SchemaSettings, schema::RootSchema, JsonSchema};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::error::ToolCallError;

/// It's highly recommended that the `JsonSchema` has descriptions for all attributes.
/// Descriptions can be defined with `#[schemars(descripion = "Hi I am an attribute")]` above each attribute
// TODO enforce at compile-time
pub trait Tool {
    type Params: Parameters;

    fn name() -> &'static str;
    fn description() -> &'static str;

    /// Call the tool.
    /// Note that returning an Err will cause it to be bubbled up. If you want the LLM to handle the error,
    /// return that error as a string.
    fn call(
        &mut self,
        parameters: Self::Params,
    ) -> impl Future<Output = Result<String, Box<dyn Error>>>;
}

pub trait Parameters: DeserializeOwned + JsonSchema {}

impl<P: DeserializeOwned + JsonSchema> Parameters for P {}

pub trait ToolGroup {
    fn tool_info(out: &mut Vec<ToolInfo>);

    fn call(
        &mut self,
        tool_call: &ToolCallFunction,
    ) -> impl Future<Output = Result<String, ToolCallError>>;
}

impl ToolGroup for () {
    fn tool_info(_: &mut Vec<ToolInfo>) {}

    async fn call(&mut self, _tool_call: &ToolCallFunction) -> Result<String, ToolCallError> {
        Err(ToolCallError::UnknownToolName)
    }
}

impl<T: Tool> ToolGroup for T {
    fn tool_info(out: &mut Vec<ToolInfo>) {
        out.push(ToolInfo::new::<_, T>())
    }

    async fn call(&mut self, tool_call: &ToolCallFunction) -> Result<String, ToolCallError> {
        if tool_call.name == T::name() {
            let p = serde_json::from_value(tool_call.arguments.clone())?;
            return Ok(serde_json::to_string(&self.call(p).await?)?);
        }

        Err(ToolCallError::UnknownToolName)
    }
}

impl<A: ToolGroup, B: ToolGroup> ToolGroup for (A, B) {
    fn tool_info(out: &mut Vec<ToolInfo>) {
        A::tool_info(out);
        B::tool_info(out);
    }

    async fn call(&mut self, arguments: &ToolCallFunction) -> Result<String, ToolCallError> {
        match self.0.call(arguments).await {
            Ok(x) => Ok(x),
            Err(ToolCallError::UnknownToolName) => self.1.call(arguments).await,
            Err(e) => Err(e),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ToolInfo {
    #[serde(rename = "type")]
    tool_type: ToolType,
    function: ToolFunctionInfo,
}

impl ToolInfo {
    fn new<P: Parameters, T: Tool<Params = P>>() -> Self {
        let mut settings = SchemaSettings::draft07();
        settings.inline_subschemas = true;
        let generator = settings.into_generator();

        let parameters = generator.into_root_schema_for::<P>();

        Self {
            tool_type: ToolType::Function,
            function: ToolFunctionInfo {
                name: T::name(),
                description: T::description(),
                parameters,
            },
        }
    }
}

#[derive(Clone, Debug, Serialize)]
enum ToolType {
    Function,
}

#[derive(Clone, Debug, Serialize)]
struct ToolFunctionInfo {
    name: &'static str,
    description: &'static str,
    parameters: RootSchema,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub function: ToolCallFunction,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCallFunction {
    name: String,
    // I don't love this (the Value)
    // But fixing it would be a big effort
    arguments: Value,
}
