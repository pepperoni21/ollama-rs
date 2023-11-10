#![allow(unused_imports)]

use ollama_rs::{
    generation::completion::{request::{GenerationRequest, FormatEnum}, GenerationResponseStream},
    Ollama,
};
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;

#[allow(dead_code)]
const PROMPT: &str = "Why is the sky blue?";

#[tokio::test]
async fn test_generation_stream() {
    let ollama = Ollama::default();

    let mut res: GenerationResponseStream = ollama
        .generate_stream(GenerationRequest::new(
            "llama2:latest".to_string(),
            PROMPT.into(),
            None
        ))
        .await
        .unwrap();

    let mut done = false;
    while let Some(res) = res.next().await {
        let res = res.unwrap();
        if res.done {
            done = true;
            break;
        }
    }

    assert!(done);
}

#[tokio::test]
async fn test_generation() {
    let ollama = Ollama::default();

    let _ = ollama
        .generate(GenerationRequest::new(
            "llama2:latest".to_string(),
            PROMPT.into(),
            None
        ))
        .await
        .unwrap();
}
#[tokio::test]
async fn test_generation_json() {
    let ollama = Ollama::default();

    let _ = ollama
        .generate_json(GenerationRequest::new(
            "llama2:latest".to_string(),
            PROMPT.into(),
            Some(FormatEnum::Json)
            
        ))
        .await
        .unwrap();
}