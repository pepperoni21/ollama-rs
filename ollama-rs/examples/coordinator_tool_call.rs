use futures_util::TryStreamExt;
use ollama_rs::{
    coordinator::Coordinator,
    generation::{
        chat::ChatMessage,
        tools::implementations::{Calculator, DDGSearcher, Scraper},
    },
    models::ModelOptions,
    Ollama,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio_stream::StreamExt;

const MODEL: &str = "qwen2.5:32b";

/// Usage:
/// cargo run --example coordinator_tool_call -- qwen3:30b
/// cargo run --example coordinator_tool_call -- qwen3:30b debug
/// cargo run --example coordinator_tool_call -- qwen3:30b no-debug
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1).fuse();
    let model = args.next().unwrap_or_else(|| MODEL.into());
    let debug = match args.next().as_deref() {
        Some("debug") => true,
        Some("no-debug") | None => false,
        Some(other) => {
            return Err(format!("Unexpected value: {other}. Just debug|no-debug is valid").into())
        }
    };

    let ollama = Ollama::default();
    let history = Vec::new();

    let mut coordinator = Coordinator::new(ollama, model, history)
        .options(ModelOptions::default().num_ctx(16384))
        .add_tool(DDGSearcher::new())
        .add_tool(Scraper::default())
        .add_tool(Calculator::default())
        .debug(debug);

    let mut commands = std::pin::pin!({
        let stdin_reader = tokio::io::BufReader::new(tokio::io::stdin());
        let line_stream = tokio_stream::wrappers::LinesStream::new(stdin_reader.lines());
        line_stream.try_take_while(|x| std::future::ready(Ok(!x.eq_ignore_ascii_case("exit"))))
    });
    let mut stdout = tokio::io::stdout();

    loop {
        stdout.write_all(b"\n> ").await?;
        stdout.flush().await?;
        let input = match commands.next().await {
            Some(Ok(x)) => x,
            Some(Err(e)) => return Err(e.into()),
            None => break,
        };

        let resp = coordinator.chat(vec![ChatMessage::user(input)]).await?;

        stdout.write_all(resp.message.content.as_bytes()).await?;
    }

    Ok(())
}
