//! Verifies that an error emitted mid-stream by the server is surfaced to the
//! caller (as `OllamaError::InternalError`) rather than being swallowed.
//!
//! Ollama terminates a streaming `/api/chat` response with an NDJSON line of the
//! shape `{"error": "..."}` when generation fails partway through (e.g. a CUDA
//! OOM). We stand up a tiny mock server that replays exactly that wire sequence
//! and assert the stream yields the real message.
#![cfg(feature = "stream")]

use ollama_rs::{
    error::OllamaError,
    generation::chat::{request::ChatMessageRequest, ChatMessage},
    Ollama,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_stream::StreamExt;

/// Spawn a one-shot mock server that returns `body` as the response body with a
/// 200 status, then closes the connection. Returns the base URL to point Ollama at.
async fn spawn_mock(body: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut sock, _) = listener.accept().await.unwrap();
        // Drain the request headers so the client's write side doesn't error.
        let mut buf = [0u8; 4096];
        let _ = sock.read(&mut buf).await;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson\r\nConnection: close\r\n\r\n{body}"
        );
        sock.write_all(response.as_bytes()).await.unwrap();
        sock.flush().await.unwrap();
    });
    format!("http://{addr}/")
}

#[tokio::test]
async fn mid_stream_error_is_surfaced() {
    // Two good chunks, then the error line Ollama sends on a mid-generation OOM.
    let body = concat!(
        r#"{"model":"m","created_at":"t","message":{"role":"assistant","content":"Hello"},"done":false}"#,
        "\n",
        r#"{"model":"m","created_at":"t","message":{"role":"assistant","content":" there"},"done":false}"#,
        "\n",
        r#"{"error":"CUDA error: out of memory"}"#,
        "\n",
    );
    let url = spawn_mock(body).await;
    let ollama = Ollama::from_url(url.parse().unwrap());

    let mut stream = ollama
        .send_chat_messages_stream(ChatMessageRequest::new(
            "m".to_string(),
            vec![ChatMessage::user("hi".to_string())],
        ))
        .await
        .expect("initial call succeeds");

    let mut contents = Vec::new();
    let mut final_err = None;
    while let Some(item) = stream.next().await {
        match item {
            Ok(resp) => contents.push(resp.message.content),
            Err(e) => {
                final_err = Some(e);
                break;
            }
        }
    }

    assert_eq!(contents, vec!["Hello".to_string(), " there".to_string()]);
    match final_err.expect("stream must yield the mid-stream error") {
        OllamaError::InternalError(e) => {
            assert_eq!(e.message, "CUDA error: out of memory");
        }
        other => panic!("expected InternalError, got {other:?}"),
    }
}
