use std::io::{stdin, stdout, Write};

use ollama_rs::{coordinator::Coordinator, generation::chat::ChatMessage, Ollama};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ollama = Ollama::default();
    let mut history = vec![];
    let mut coordinator = Coordinator::new(&mut ollama, &mut history, "llama3.2".to_string());

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
            .chat(ChatMessage::user(input.to_string()))
            .await?;

        println!("{}", resp.message.content);
    }

    dbg!(history);

    Ok(())
}
