use futures_util::TryStreamExt;
use ollama_rs::{
    generation::chat::{request::ChatMessageRequest, ChatMessage},
    Ollama,
};
use tokio::io::{stdout, AsyncBufReadExt, AsyncWriteExt};
use tokio_stream::StreamExt;

const MODEL: &str = "llama3.2:latest";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1).fuse();
    let model = args.next().unwrap_or_else(|| MODEL.to_string());

    let ollama = Ollama::default();
    let mut history = vec![];
    let mut stdout = stdout();
    let mut commands = std::pin::pin!({
        let stdin_reader = tokio::io::BufReader::new(tokio::io::stdin());
        let line_stream = tokio_stream::wrappers::LinesStream::new(stdin_reader.lines());
        line_stream.try_take_while(|x| std::future::ready(Ok(!x.eq_ignore_ascii_case("exit"))))
    });

    loop {
        stdout.write_all(b"\n> ").await?;
        stdout.flush().await?;
        let input = match commands.next().await {
            Some(Ok(x)) => x,
            Some(Err(e)) => return Err(e.into()),
            None => break,
        };

        let user_message = ChatMessage::user(input.to_string());
        let request = ChatMessageRequest::new(model.clone(), vec![user_message]);
        let response = ollama
            .send_chat_messages_with_history(&mut history, request)
            .await?;

        let assistant_message = response.message.content;
        stdout.write_all(assistant_message.as_bytes()).await?;
    }

    // Display whole history of messages
    dbg!(&history);

    Ok(())
}
