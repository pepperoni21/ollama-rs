use base64::Engine;
use ollama_rs::{
    generation::{
        completion::{request::GenerationRequest, GenerationResponse},
        images::Image,
    },
    Ollama,
};
use reqwest::get;
use tokio::runtime::Runtime;

const IMAGE_URL: &str = "https://images.pexels.com/photos/1054655/pexels-photo-1054655.jpeg";
const PROMPT: &str = "Describe this image";

fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        // Download the image and encode it to base64
        let bytes = match download_image(IMAGE_URL).await {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Failed to download image: {}", e);
                return;
            }
        };
        let base64_image = base64::engine::general_purpose::STANDARD.encode(&bytes);

        // Create an Image struct from the base64 string
        let image = Image::from_base64(&base64_image);

        // Create a GenerationRequest with the model and prompt, adding the image
        let request =
            GenerationRequest::new("llava:latest".to_string(), PROMPT.to_string()).add_image(image);

        // Send the request to the model and get the response
        let response = match send_request(request).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to get response: {}", e);
                return;
            }
        };

        // Print the response
        println!("{}", response.response);
    });
}

// Function to download the image
async fn download_image(url: &str) -> Result<Vec<u8>, reqwest::Error> {
    let response = get(url).await?;
    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}

// Function to send the request to the model
async fn send_request(
    request: GenerationRequest,
) -> Result<GenerationResponse, Box<dyn std::error::Error>> {
    let ollama = Ollama::default();
    let response = ollama.generate(request).await?;
    Ok(response)
}
