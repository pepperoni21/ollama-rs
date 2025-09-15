use calc::Context;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::generation::tools::Tool;

#[derive(Deserialize, JsonSchema)]
pub struct Params {
    #[schemars(
        description = "The mathematical expression to calculator. General formatting guidelines:
- Use `*` for multiplication
- Use `**` for exponents
- Be sure to use parantheses for more complicated expressions"
    )]
    expression: String,
}

#[derive(Default)]
pub struct Calculator {}

impl Tool for Calculator {
    type Params = Params;

    fn name() -> &'static str {
        "calculator"
    }

    fn description() -> &'static str {
        "Evaluates an arbitrary mathematical expression. Can only evaluate one expression at a time."
    }

    async fn call(
        &mut self,
        parameters: Self::Params,
    ) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
        let mut ctx: Context<f64> = Context::default();

        let res = match ctx.evaluate(&parameters.expression) {
            Ok(x) => format!("{x}"),
            Err(e) => format!("Calc evaluation error: {e:?}"),
        };

        Ok(res)
    }
}
