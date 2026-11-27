//! Verifies that streamed chat records survive HTTP chunk boundaries that split
//! a multi-byte UTF-8 character.
//!
//! Ollama streams `/api/chat` responses as newline-delimited JSON. reqwest may
//! deliver the body in chunks that cut straight through a code point (for
//! example, the two bytes of `é`). The client must buffer raw bytes until a full
//! record is assembled and then decode it, otherwise the record is silently
//! dropped. We stand up a tiny mock server that sends one record across two
//! chunked-encoding frames, splitting inside the character, and assert the reply
//! is still returned intact.
#![cfg(feature = "stream")]

use ollama_rs::{
    generation::chat::{request::ChatMessageRequest, ChatMessage},
    Ollama,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_stream::StreamExt;

#[tokio::test]
async fn streamed_chat_preserves_utf8_split_between_http_chunks() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();

        // Drain the request headers so the client's write side doesn't error.
        let mut request = [0; 4096];
        let _ = socket.read(&mut request).await.unwrap();

        socket
            .write_all(
                b"HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\ncontent-type: application/x-ndjson\r\n\r\n",
            )
            .await
            .unwrap();

        // One record whose `é` (two UTF-8 bytes) is split across two frames.
        let line = r#"{"model":"test","created_at":"2026-01-01T00:00:00Z","message":{"role":"assistant","content":"café"},"done":true}"#;
        let split = line.find('é').unwrap() + 1;
        for part in [&line.as_bytes()[..split], &line.as_bytes()[split..]] {
            socket
                .write_all(format!("{:x}\r\n", part.len()).as_bytes())
                .await
                .unwrap();
            socket.write_all(part).await.unwrap();
            socket.write_all(b"\r\n").await.unwrap();
        }
        socket.write_all(b"0\r\n\r\n").await.unwrap();
    });

    let ollama = Ollama::try_new(format!("http://{addr}")).unwrap();
    let request = ChatMessageRequest::new("test".into(), vec![ChatMessage::user("hi".into())]);
    let mut stream = ollama.send_chat_messages_stream(request).await.unwrap();

    let response = stream
        .next()
        .await
        .expect("received a chat response")
        .unwrap();
    assert_eq!(response.message.content, "café");
    assert!(response.done);
    assert!(stream.next().await.is_none());

    server.await.unwrap();
}
