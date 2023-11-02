#![allow(unused_imports)]

use tokio_stream::StreamExt;
use tokio::io::AsyncWriteExt;

use crate::{Ollama, generation::{request::GenerationRequest, GenerationResponseStream}};

#[allow(dead_code)]
const PROMPT: &'static str = "Why is the sky blue?";

#[tokio::test]
async fn test_generation_stream() {
    let ollama = Ollama::default();

    let mut res: GenerationResponseStream = ollama.generate_stream(GenerationRequest::new("llama2:latest".to_string(), PROMPT.into())).await.unwrap();

    let mut stdout = tokio::io::stdout();
    while let Some(res) = res.next().await {
        let res = res.unwrap();
        stdout.write(res.response.as_bytes()).await.unwrap();
        stdout.flush().await.unwrap();
    }
}

#[tokio::test]
async fn test_generation() {
    let ollama = Ollama::default();

    let res = ollama.generate(GenerationRequest::new("llama2:latest".to_string(), PROMPT.into())).await.unwrap();

    println!("{}", res.response);
}