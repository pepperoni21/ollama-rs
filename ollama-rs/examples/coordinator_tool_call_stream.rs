use std::sync::Arc;
use std::sync::Mutex;
use std::io::Write;
use std::io::stdin;
use std::io::stdout;

use tokio_stream::StreamExt;

use ollama_rs::{
    coordinator::Coordinator,
    generation::{
        chat::ChatMessage,
        options::GenerationOptions,
        tools::implementations::{Calculator, DDGSearcher, Scraper},
    },
    Ollama,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() > 2 || (args.get(1).is_some() && args[1] != "-d") {
        eprintln!("Usage: {} [-d] (to enable debugging)", args[0],);
        return Ok(());
    }

    let debug = args.get(1).is_some();

    let ollama = Ollama::default();
    let history = Arc::new(Mutex::new(vec![]));
    let tools = (DDGSearcher::new(), (Scraper {}, Calculator {}));

    let mut coordinator =
        Coordinator::new_with_tools(ollama, "qwen2.5-coder:14b".to_string(), history, tools)
            .options(GenerationOptions::default().num_ctx(16384))
            .debug(debug);

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

        let mut stream = coordinator
            .chat_stream(vec![ChatMessage::user(input.to_string())])
            .await?;

        let mut response = String::new();
        while let Some(Ok(res)) = stream.next().await {
            stdout.write_all(res.message.content.as_bytes())?;
            stdout.flush()?;
            response += res.message.content.as_str();
        }
    }

    Ok(())
}
