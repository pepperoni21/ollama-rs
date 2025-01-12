use ollama_rs::{
    generation::chat::{request::ChatMessageRequest, ChatMessage, ChatMessageResponseStream},
    Ollama,
};
use std::sync::{Arc, Mutex};
use tokio::io::{stdout, AsyncWriteExt};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ollama = Ollama::default();
    let history = Arc::new(Mutex::new(vec![]));
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

        let mut stream: ChatMessageResponseStream = ollama
            .send_chat_messages_with_history_stream(
                history.clone(),
                ChatMessageRequest::new(
                    "llama3.2:latest".to_string(),
                    vec![ChatMessage::user(input.to_string())],
                ),
            )
            .await?;

        let mut response = String::new();
        while let Some(Ok(res)) = stream.next().await {
            stdout.write_all(res.message.content.as_bytes()).await?;
            stdout.flush().await?;
            response += res.message.content.as_str();
        }
    }

    dbg!(&history.lock().unwrap());

    Ok(())
}
