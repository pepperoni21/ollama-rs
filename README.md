# Ollama-rs

### A simple and easy to use library for interacting with the Ollama API.

It was made following the [Ollama API](https://github.com/jmorganca/ollama/blob/main/docs/api.md) documentation.

## Installation

### Add ollama-rs to your Cargo.toml

```toml
[dependencies]
ollama-rs = "0.1.8"
```

### Initialize Ollama

```rust
// By default it will connect to localhost:11434
let ollama = Ollama::default();

// For custom values:
let ollama = Ollama::new("http://localhost".to_string(), 11434);
```

## Usage

Feel free to check the [Chatbot example](examples/basic_chatbot.rs) that shows how to use the library to create a simple chatbot in less than 50 lines of code.

_These examples use poor error handling for simplicity, but you should handle errors properly in your code._

### Completion generation

```rust
let model = "llama2:latest".to_string();
let prompt = "Why is the sky blue?".to_string();

let res = ollama.generate(GenerationRequest::new(model, prompt)).await;

if let Ok(res) = res {
    println!("{}", res.response);
}
```

**OUTPUTS:** _The sky appears blue because of a phenomenon called Rayleigh scattering..._

### Completion generation (streaming)

_Requires the `stream` feature._

```rust
let model = "llama2:latest".to_string();
let prompt = "Why is the sky blue?".to_string();

let mut stream = ollama.generate_stream(GenerationRequest::new(model, prompt)).await.unwrap();

let mut stdout = tokio::io::stdout();
while let Some(res) = stream.next().await {
    let responses = res.unwrap();
    for resp in responses {
        stdout.write(resp.response.as_bytes()).await.unwrap();
        stdout.flush().await.unwrap();
    }
}
```

Same output as above but streamed.

### Completion generation (passing options to the model)

```rust
let model = "llama2:latest".to_string();
let prompt = "Why is the sky blue?".to_string();

let options = GenerationOptions::default()
    .temperature(0.2)
    .repeat_penalty(1.5)
    .top_k(25)
    .top_p(0.25);

let res = ollama.generate(GenerationRequest::new(model, prompt).options(options)).await;

if let Ok(res) = res {
    println!("{}", res.response);
}
```

**OUTPUTS:** _1. Sun emits white sunlight: The sun consists primarily ..._

### List local models

```rust
let res = ollama.list_local_models().await.unwrap();
```

_Returns a vector of `Model` structs._

### Show model information

```rust
let res = ollama.show_model_info("llama2:latest".to_string()).await.unwrap();
```

_Returns a `ModelInfo` struct._

### Create a model

```rust
let res = ollama.create_model(CreateModelRequest::path("model".into(), "/tmp/Modelfile.example".into())).await.unwrap();
```

_Returns a `CreateModelStatus` struct representing the final status of the model creation._

### Create a model (streaming)

_Requires the `stream` feature._

```rust
let mut res = ollama.create_model_stream(CreateModelRequest::path("model".into(), "/tmp/Modelfile.example".into())).await.unwrap();

while let Some(res) = res.next().await {
    let res = res.unwrap();
    // Handle the status
}
```

_Returns a `CreateModelStatusStream` that will stream every status update of the model creation._

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

_Returns a `GenerateEmbeddingsResponse` struct containing the embeddings (a vector of floats)._
