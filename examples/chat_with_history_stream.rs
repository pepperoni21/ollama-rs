use ollama_rs::{
    generation::chat::{request::ChatMessageRequest, ChatMessage},
    Ollama,
};
use tokio::io::{stdout, AsyncWriteExt};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ollama = Ollama::new_default_with_history_async(30);

    let mut stdout = stdout();

    let chat_id = "default".to_string();

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

        let mut stream = ollama
            .send_chat_messages_with_history_stream(
                ChatMessageRequest::new("llama2:latest".to_string(), vec![user_message]),
                chat_id.clone(),
            )
            .await?;

        let mut response = String::new();
        while let Some(Ok(res)) = stream.next().await {
            if let Some(assistant_message) = res.message {
                stdout
                    .write_all(assistant_message.content.as_bytes())
                    .await?;
                stdout.flush().await?;
                response += assistant_message.content.as_str();
            }
        }
    }

    // Display whole history of messages
    dbg!(
        &ollama
            .get_messages_history_async("default".to_string())
            .await
    );

    Ok(())
}
