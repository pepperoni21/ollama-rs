use schemars::{gen::SchemaSettings, schema::RootSchema, JsonSchema};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::error::ToolCallError;

/// It's highly recommended that the JsonSchema has descriptions for all attributes
// TODO enforce at compile-time
pub trait Tool {
    type P: Parameter;

    fn name() -> &'static str;
    fn description() -> &'static str;

    // TODO I think a string may be too limiting. Should prob just be serializeable?
    fn call(&mut self, parameters: Self::P) -> String;
}

pub trait Parameter: DeserializeOwned + JsonSchema {}

impl<P: DeserializeOwned + JsonSchema> Parameter for P {}

pub trait ToolGroup {
    fn tool_info() -> Vec<ToolInfo>;

    fn call(&mut self, arguments: ToolCallFunction) -> Result<String, ToolCallError>;
}

impl ToolGroup for () {
    fn tool_info() -> Vec<ToolInfo> {
        vec![]
    }

    fn call(&mut self, _arguments: ToolCallFunction) -> Result<String, ToolCallError> {
        Err(ToolCallError::UnknownToolName)
    }
}

impl<T: Tool> ToolGroup for T {
    fn tool_info() -> Vec<ToolInfo> {
        vec![ToolInfo::new::<_, T>()]
    }

    fn call(&mut self, tool_call: ToolCallFunction) -> Result<String, ToolCallError> {
        if tool_call.name == T::name() {
            let p = serde_json::from_value(tool_call.arguments)?;
            return Ok(self.call(p));
        }

        return Err(ToolCallError::UnknownToolName);
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ToolInfo {
    #[serde(rename = "type")]
    tool_type: ToolType,
    function: ToolFunctionInfo,
}

impl ToolInfo {
    fn new<P: Parameter, T: Tool<P = P>>() -> Self {
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

macro_rules! tool_group {
    ($($tool:ident),*) => {
        impl<$($tool: Tool),*> ToolGroup for ($($tool,)*)
        {
            fn tool_info() -> Vec<ToolInfo> {
                vec![
		    $(
                        ToolInfo::new::<_, $tool>(),
		    )*
                ]
            }

	    fn call(&mut self, tool_call: ToolCallFunction) -> Result<String, ToolCallError> {
		todo!()
	    }
        }
    }
}

tool_group!(A);
tool_group!(A, B);
tool_group!(A, B, C);
tool_group!(A, B, C, D);
tool_group!(A, B, C, D, E);
tool_group!(A, B, C, D, E, F);
tool_group!(A, B, C, D, E, F, G);
tool_group!(A, B, C, D, E, F, G, H);
tool_group!(A, B, C, D, E, F, G, H, I);
tool_group!(A, B, C, D, E, F, G, H, I, J);
tool_group!(A, B, C, D, E, F, G, H, I, J, K);
tool_group!(A, B, C, D, E, F, G, H, I, J, K, L);
tool_group!(A, B, C, D, E, F, G, H, I, J, K, L, M);
tool_group!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
tool_group!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
tool_group!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
tool_group!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
tool_group!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
tool_group!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
tool_group!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
tool_group!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
tool_group!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
tool_group!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
tool_group!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
tool_group!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
tool_group!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
tool_group!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA);
tool_group!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB);
tool_group!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC
);
tool_group!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD
);
tool_group!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD,
    AE
);
tool_group!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD,
    AE, AF
);
tool_group!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD,
    AE, AF, AG
);
tool_group!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD,
    AE, AF, AG, AH
);
tool_group!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD,
    AE, AF, AG, AH, AI
);
tool_group!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD,
    AE, AF, AG, AH, AI, AJ
);
tool_group!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD,
    AE, AF, AG, AH, AI, AJ, AK
);
tool_group!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD,
    AE, AF, AG, AH, AI, AJ, AK, AL
);
tool_group!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD,
    AE, AF, AG, AH, AI, AJ, AK, AL, AM
);
tool_group!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD,
    AE, AF, AG, AH, AI, AJ, AK, AL, AM, AN
);
tool_group!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD,
    AE, AF, AG, AH, AI, AJ, AK, AL, AM, AN, AO
);
tool_group!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD,
    AE, AF, AG, AH, AI, AJ, AK, AL, AM, AN, AO, AP
);
tool_group!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD,
    AE, AF, AG, AH, AI, AJ, AK, AL, AM, AN, AO, AP, AQ
);
tool_group!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD,
    AE, AF, AG, AH, AI, AJ, AK, AL, AM, AN, AO, AP, AQ, AR
);
tool_group!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD,
    AE, AF, AG, AH, AI, AJ, AK, AL, AM, AN, AO, AP, AQ, AR, AS
);
tool_group!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD,
    AE, AF, AG, AH, AI, AJ, AK, AL, AM, AN, AO, AP, AQ, AR, AS, AT
);
tool_group!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD,
    AE, AF, AG, AH, AI, AJ, AK, AL, AM, AN, AO, AP, AQ, AR, AS, AT, AU
);
