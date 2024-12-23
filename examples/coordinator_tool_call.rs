use std::io::{stdin, stdout, Write};

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
    let mut ollama = Ollama::default();
    let mut history = vec![];
    let tools = (DDGSearcher::new(), (Scraper {}, Calculator {}));

    let mut coordinator =
        Coordinator::new_with_tools(&mut ollama, "qwen2.5:32b".to_string(), &mut history, tools)
            .options(GenerationOptions::default().num_ctx(16384));

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
