use ollama_rs::{
    generation::chat::{request::ChatMessageRequest, ChatMessage},
    Ollama,
};
use tokio_stream::StreamExt;

#[allow(dead_code)]
const PROMPT: &str = "Why is the sky blue?";

#[tokio::test]
async fn test_send_chat_messages_stream() {
    let ollama = Ollama::default();

    let messages = vec![ChatMessage::user(PROMPT.to_string())];
    let mut res = ollama
        .send_chat_messages_stream(ChatMessageRequest::new(
            "llama2:latest".to_string(),
            messages,
        ))
        .await
        .unwrap();

    let mut done = false;
    while let Some(res) = res.next().await {
        let res = res.unwrap();
        dbg!(&res);
        if res.done {
            done = true;
            break;
        }
    }

    assert!(done);
}

#[tokio::test]
async fn test_send_chat_messages() {
    let ollama = Ollama::default();

    let messages = vec![ChatMessage::user(PROMPT.to_string())];
    let res = ollama
        .send_chat_messages(ChatMessageRequest::new(
            "llama2:latest".to_string(),
            messages,
        ))
        .await
        .unwrap();
    dbg!(&res);

    assert!(res.done);
}
