# Ollama-rs
### A simple and easy to use library for interacting with Ollama servers.
It was made following the [Ollama API](https://github.com/jmorganca/ollama/blob/main/docs/api.md) documentation.

## Installation
### Add ollama-rs to your Cargo.toml
```toml
[dependencies]
ollama-rs = "0.1.4"
```
### Initialize Ollama
```rust
// By default it will connect to localhost:11434
let ollama = Ollama::default();

// For custom values:
let ollama = Ollama::new("http://localhost".to_string(), 11434);
```

## Usage
Feel free to check the [Chatbot example](examples/chatbot.rs) that shows how to use the library to create a simple chatbot in less than 50 lines of code.

*These examples use poor error handling for simplicity, but you should handle errors properly in your code.*
### Completion generation
```rust
let model = "llama2:latest".to_string();
let prompt = "Why is the sky blue?".to_string();

let res = ollama.generate(GenerationRequest::new(model, prompt)).await;

if let Ok(res) = res {
    println!("{}", res.response);
}
```
**OUTPUTS:** *The sky appears blue because of a phenomenon called Rayleigh scattering...*
### Completion generation (streaming)
*Requires the `stream` feature.*
```rust
let model = "llama2:latest".to_string();
let prompt = "Why is the sky blue?".to_string();

let mut stream = ollama.generate_stream(GenerationRequest::new(model, prompt)).await.unwrap();

let mut stdout = tokio::io::stdout();
while let Some(res) = stream.next().await {
    let res = res.unwrap();
    stdout.write(res.response.as_bytes()).await.unwrap();
    stdout.flush().await.unwrap();
}
```
Same output as above but streamed.
### List local models
```rust
let res = ollama.list_local_models().await.unwrap();
```
*Returns a vector of `Model` structs.*
### Show model information
```rust
let res = ollama.show_model_info("llama2:latest".to_string()).await.unwrap();
```
*Returns a `ModelInfo` struct.*
### Create a model
```rust
let res = ollama.create_model(CreateModelRequest::path("model".into(), "/tmp/Modelfile.example".into())).await.unwrap();
```
*Returns a `CreateModelStatus` struct representing the final status of the model creation.*
### Create a model (streaming)
*Requires the `stream` feature.*
```rust
let mut res = ollama.create_model_stream(CreateModelRequest::path("model".into(), "/tmp/Modelfile.example".into())).await.unwrap();

while let Some(res) = res.next().await {
    let res = res.unwrap();
    // Handle the status
}
```
*Returns a `CreateModelStatusStream` that will stream every status update of the model creation.*
### Copy a model
```rust
let _ = ollama.copy_model("mario".into(), "mario_copy".into()).await.unwrap();
```
### Delete a model
```rust
let _ = ollama.delete_model("mario_copy".into()).await.unwrap();
```
### Generate embeddings
```rust
let prompt = "Why is the sky blue?".to_string();
let res = ollama.generate_embeddings("llama2:latest".to_string(), prompt, None).await.unwrap();
```
*Returns a `GenerateEmbeddingsResponse` struct containing the embeddings (a vector of floats).*
