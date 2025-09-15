use std::error::Error;

use base64::Engine;
use ollama_rs::{
    generation::{
        completion::{request::GenerationRequest, GenerationResponse},
        images::Image,
    },
    Ollama,
};
use reqwest::get;

const IMAGE_URL: &str = "https://images.pexels.com/photos/1054655/pexels-photo-1054655.jpeg";
const PROMPT: &str = "Describe this image";
const MODEL: &str = "llava:latest";

/// Usage:
/// cargo run --example images_to_ollama
/// cargo run --example images_to_ollama -- https://assets.canarymedia.com/content/uploads/Alex-honnold-lead-resized.jpg
/// cargo run --example images_to_ollama -- https://assets.canarymedia.com/content/uploads/Alex-honnold-lead-resized.jpg "What color is the shirt?"
/// cargo run --example images_to_ollama -- https://assets.canarymedia.com/content/uploads/Alex-honnold-lead-resized.jpg "Is this person climbing free solo based on his gear?" llava:34b
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1).fuse();
    let image_url = args.next().unwrap_or_else(|| IMAGE_URL.into());
    let prompt = args.next().unwrap_or_else(|| PROMPT.into());
    let model = args.next().unwrap_or_else(|| MODEL.into());

    let image = download_image(&image_url).await?;
    let request = GenerationRequest::new(model, prompt).add_image(image);
    let response = send_request(request).await?;

    println!("{}", response.response);
    Ok(())
}

async fn download_image(url: &str) -> Result<Image, reqwest::Error> {
    let response = get(url).await?;
    let bytes = response.bytes().await?;
    let base64_image = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(Image::from_base64(&base64_image))
}

async fn send_request(
    request: GenerationRequest<'_>,
) -> Result<GenerationResponse, Box<dyn Error>> {
    let ollama = Ollama::default();
    let response = ollama.generate(request).await?;
    Ok(response)
}
