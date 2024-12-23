use std::{
    error::Error,
    io::{stdin, stdout, Write},
};

use ollama_rs::{
    coordinator::Coordinator,
    generation::{chat::ChatMessage, functions::Tool},
    Ollama,
};
use schemars::JsonSchema;
use serde::Deserialize;

struct SearchTool {}

#[derive(Deserialize, JsonSchema)]
struct SearchToolParameters {
    query: String,
}

impl Tool for SearchTool {
    type P = SearchToolParameters;

    fn name() -> &'static str {
        "search_engine_tool"
    }

    fn description() -> &'static str {
        "Searches a search engine on the Internet for the given query"
    }

    fn call(&mut self, _parameters: Self::P) -> Result<String, Box<dyn Error>> {
        Ok("Intel stock price is $523.52".to_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ollama = Ollama::default();
    let mut history = vec![];
    let tools = SearchTool {};
    let mut coordinator =
        Coordinator::new_with_tools(&mut ollama, "qwen2.5:32b".to_string(), &mut history, tools);

    let stdin = stdin();
    let mut stdout = stdout();
    loop {
        stdout.write_all(b"\n> ")?;
        stdout.flush()?;

        let mut input = String::new();
        stdin.read_line(&mut input)?;

        let input = input.trim_end();
        if input.eq_ignore_ascii_case("exit") {
            break;
        }

        let resp = coordinator
            .chat(vec![ChatMessage::user(input.to_string())])
            .await?;

        println!("{}", resp.message.content);
    }

    dbg!(history);

    Ok(())
}
