use ollama_rs::{
    generation::chat::{request::ChatMessageRequest, ChatMessage},
    Ollama,
};
use tokio::io::{stdout, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ollama = Ollama::default();
    let mut history = vec![];
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

        let result = ollama
            .send_chat_messages_with_history(
                &mut history,
                ChatMessageRequest::new("llama3.2:latest".to_string(), vec![user_message]),
            )
            .await?;

        let assistant_message = result.message.content;
        stdout.write_all(assistant_message.as_bytes()).await?;
        stdout.flush().await?;
    }

    // Display whole history of messages
    dbg!(&history);

    Ok(())
}
