use std::io::{stdin, stdout, Write};

use ollama_rs::{
    coordinator::Coordinator,
    generation::{
        chat::ChatMessage,
        tools::implementations::{Calculator, DDGSearcher, Scraper},
    },
    models::ModelOptions,
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
    let history = vec![];
    let tools = (DDGSearcher::new(), (Scraper {}, Calculator {}));

    let mut coordinator =
        Coordinator::new_with_tools(ollama, "qwen2.5:32b".to_string(), history, tools)
            .options(ModelOptions::default().num_ctx(16384))
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

        let resp = coordinator
            .chat(vec![ChatMessage::user(input.to_string())])
            .await?;

        println!("{}", resp.message.content);
    }

    Ok(())
}
