use futures_util::TryStreamExt;
use ollama_rs::{
    generation::chat::{request::ChatMessageRequest, ChatMessage, ChatMessageResponseStream},
    Ollama,
};
use tokio::io::{stdout, AsyncBufReadExt, AsyncWriteExt};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ollama = Ollama::default();

    let mut stdout = stdout();
    let mut commands = std::pin::pin!({
        let stdin_reader = tokio::io::BufReader::new(tokio::io::stdin());
        let line_stream = tokio_stream::wrappers::LinesStream::new(stdin_reader.lines());
        line_stream.try_take_while(|x| std::future::ready(Ok(!x.eq_ignore_ascii_case("exit"))))
    });
    let mut messages: Vec<ChatMessage> = vec![];

    loop {
        stdout.write_all(b"\n> ").await?;
        stdout.flush().await?;

        let input = match commands.next().await {
            Some(Ok(x)) => x,
            Some(Err(e)) => return Err(e.into()),
            None => break,
        };

        let user_message = ChatMessage::user(input.to_string());
        messages.push(user_message);

        let mut stream: ChatMessageResponseStream = ollama
            .send_chat_messages_stream(ChatMessageRequest::new(
                "llama2:latest".to_string(),
                messages.clone(),
            ))
            .await?;

        let mut response = String::new();
        while let Some(res) = stream.next().await {
            let res = res.map_err(|_| "Streaming error")?;
            stdout.write_all(res.message.content.as_bytes()).await?;
            stdout.flush().await?;
            response += res.message.content.as_str();
        }
        messages.push(ChatMessage::assistant(response));
    }

    Ok(())
}
