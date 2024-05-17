use ollama_rs::generation::functions::pipelines::nous_hermes::request::NousFunctionCall;
use ollama_rs::{
    generation::chat::ChatMessage,
    generation::functions::tools::{DDGSearcher, Scraper},
    generation::functions::{FunctionCallRequest, OpenAIFunctionCall},
    Ollama,
};
use std::sync::Arc;
use tokio::io::{stdout, AsyncWriteExt};

#[tokio::test]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ollama = Ollama::new_default_with_history(30);
    let scraper_tool = Arc::new(Scraper {});
    let ddg_search_tool = Arc::new(DDGSearcher::new());
    //adrienbrault/nous-hermes2pro:Q8_0  "openhermes:latest"
    let mut stdout = stdout();

    loop {
        stdout.write_all(b"\n> ").await?;
        stdout.flush().await?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        let input = input.trim_end();
        if input.eq_ignore_ascii_case("exit") {
            break;
        }

        let user_message = ChatMessage::user(input.to_string());

        let parser = Arc::new(NousFunctionCall {});
        let result = ollama
            .send_function_call(
                FunctionCallRequest::new(
                    "adrienbrault/nous-hermes2pro:Q8_0".to_string(),
                    vec![scraper_tool.clone(), ddg_search_tool.clone()],
                    vec![user_message.clone()],
                ),
                parser.clone(),
            )
            .await?;

        if let Some(message) = result.message {
            stdout.write_all(message.content.as_bytes()).await?;
        }

        stdout.flush().await?;
    }

    // Display whole history of messages
    dbg!(&ollama.get_messages_history("default".to_string()));

    Ok(())
}
