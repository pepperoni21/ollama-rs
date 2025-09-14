use futures_util::TryStreamExt;
use ollama_rs::{coordinator::Coordinator, generation::chat::ChatMessage, Ollama};
use tokio::io::{stdout, AsyncBufReadExt, AsyncWriteExt};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ollama = Ollama::default();
    let history = vec![];
    let mut coordinator = Coordinator::new(ollama, "llama3.2".to_string(), history);

    let mut commands = std::pin::pin!({
        let stdin_reader = tokio::io::BufReader::new(tokio::io::stdin());
        let line_stream = tokio_stream::wrappers::LinesStream::new(stdin_reader.lines());
        line_stream.try_take_while(|x| std::future::ready(Ok(!x.eq_ignore_ascii_case("exit"))))
    });
    let mut stdout = stdout();
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
