//! Verifies that `send_chat_messages_with_history_stream` accumulates tool
//! calls from every streamed chunk, not just the final one.
//!
//! Ollama can emit `tool_calls` in an intermediate `done: false` chunk, followed
//! by a final `done: true` chunk with empty content and no tool calls. The
//! history helper must gather calls across all chunks in arrival order so the
//! assistant message saved for the next turn retains them. We stand up a tiny
//! mock server that emits a call in each of two chunks and assert the stored
//! history contains both.
#![cfg(feature = "stream")]

use std::sync::{Arc, Mutex};

use ollama_rs::{
    generation::chat::{request::ChatMessageRequest, ChatMessage},
    Ollama,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_stream::StreamExt;

#[tokio::test]
async fn streamed_history_preserves_tool_calls_from_all_messages() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();

        // Drain the request headers so the client's write side doesn't error.
        let mut request = Vec::new();
        let mut buffer = [0; 1024];
        loop {
            let read = socket.read(&mut buffer).await.unwrap();
            if read == 0 {
                break;
            }

            request.extend_from_slice(&buffer[..read]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }

        let body = concat!(
            r#"{"model":"test","created_at":"2026-01-01T00:00:00Z","message":{"role":"assistant","content":"hello ","tool_calls":[{"function":{"name":"first_tool","arguments":{}}}]},"done":false}"#,
            "\n",
            r#"{"model":"test","created_at":"2026-01-01T00:00:00Z","message":{"role":"assistant","content":"world","tool_calls":[{"function":{"name":"test_tool","arguments":{"city":"Paris"}}}]},"done":true}"#,
            "\n",
        );
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/x-ndjson\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            body.len(),
            body
        );

        socket.write_all(response.as_bytes()).await.unwrap();
    });

    let ollama = Ollama::try_new(format!("http://{addr}")).unwrap();
    let history = Arc::new(Mutex::new(Vec::new()));
    let request = ChatMessageRequest::new(
        "test".to_string(),
        vec![ChatMessage::user("use the tool".to_string())],
    );

    let mut stream = ollama
        .send_chat_messages_with_history_stream(history.clone(), request)
        .await
        .unwrap();

    let mut final_chunk = None;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        if chunk.done {
            final_chunk = Some(chunk);
            break;
        }
    }

    server.await.unwrap();

    let final_chunk = final_chunk.unwrap();
    assert_eq!(final_chunk.message.content, "hello world");
    assert_eq!(final_chunk.message.tool_calls.len(), 2);
    assert_eq!(
        final_chunk.message.tool_calls[0].function.name,
        "first_tool"
    );
    assert_eq!(final_chunk.message.tool_calls[1].function.name, "test_tool");

    let history = history.lock().unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[1].content, "hello world");
    assert_eq!(history[1].tool_calls.len(), 2);
    assert_eq!(history[1].tool_calls[0].function.name, "first_tool");
    assert_eq!(history[1].tool_calls[1].function.name, "test_tool");
}
