#![allow(unused_imports)]
use base64::Engine;
use futures_util::{stream::Stream, StreamExt, TryStreamExt};
use ollama_rs::{
    generation::{
        completion::{request::GenerationRequest, GenerationResponseStream},
        images::Image,
    },
    Ollama,
};
use tokio::io::AsyncWriteExt;

#[allow(dead_code)]
const PROMPT: &str = "Why is the sky blue?";

#[tokio::test]
async fn test_generation_stream() -> Result<(), Box<dyn std::error::Error>> {
    let ollama = Ollama::default();

    let mut res: GenerationResponseStream = ollama
        .generate_stream(GenerationRequest::new("llama2:latest".to_string(), PROMPT))
        .await
        .unwrap();

    let mut done = false;
    while let Some(res) = res.try_next().await? {
        dbg!(&res);
        if res.done {
            done = true;
            break;
        }
    }

    assert!(done);
    Ok(())
}

#[tokio::test]
async fn test_generation() {
    let ollama = Ollama::default();

    let res = ollama
        .generate(GenerationRequest::new("llama2:latest".to_string(), PROMPT))
        .await
        .unwrap();
    dbg!(res);
}

const IMAGE_URL: &str = "https://images.pexels.com/photos/1054655/pexels-photo-1054655.jpeg";

#[tokio::test]
async fn test_generation_with_images() {
    let ollama = Ollama::default();

    let bytes = reqwest::get(IMAGE_URL)
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let base64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let image = Image::from_base64(&base64);

    let res = ollama
        .generate(
            GenerationRequest::new(
                "llava:latest".to_string(),
                "What can we see in this image?".to_string(),
            )
            .add_image(image),
        )
        .await
        .unwrap();
    dbg!(res);
}
