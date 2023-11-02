# Ollama-rs
### A simple and easy to use library for interacting with Ollama servers.
It was made following the [Ollama API](https://github.com/jmorganca/ollama/blob/main/docs/api.md) documentation.

## Installation
### Add ollama-rs to your Cargo.toml
```toml
[dependencies]
ollama-rs = { git = "https://github.com/pepperoni21/ollama-rs" }
```

## Usage
### Completion generation
```rust
let ollama = Ollama::default(); // or Ollama::new(HOST, PORT) for custom values

let model = "llama2:latest".to_string();
let prompt = "Why is the sky blue?".to_string();

let res = ollama.generate(GenerationRequest::new(model, prompt)).await;

if let Ok(res) = res {
    println!("{}", res.response);
}
```
**OUTPUTS:** *The sky appears blue because of a phenomenon called Rayleigh scattering...*
### Completion generation (stream)
```rust
let ollama = Ollama::default(); // or Ollama::new(HOST, PORT) for custom values

let model = "llama2:latest".to_string();
let prompt = "Why is the sky blue?".to_string();

let mut stream = ollama.generate_stream(GenerationRequest::new(model, prompt)).await.unwrap(); // bad error handling for example purposes

let mut stdout = tokio::io::stdout();
// Requires tokio_stream
while let Some(res) = stream.next().await {
    let res = res.unwrap();
    stdout.write(res.response.as_bytes()).await.unwrap();
    stdout.flush().await.unwrap();
}
```
Same output as above but streamed.
### List local models
```rust
let ollama = Ollama::default(); // or Ollama::new(HOST, PORT) for custom values

let res = ollama.list_local_models().await.unwrap();
println!("{:#?}", res);
```
### Show model information
```rust
let ollama = Ollama::default(); // or Ollama::new(HOST, PORT) for custom values

let res = ollama.show_model_info("llama2:latest".to_string()).await.unwrap();
println!("{:#?}", res);
```

## TODO
- [x] Completion generation (single response)
- [x] Completion generation (streaming)
- [x] Add usage for completion generation
- [x] Better error handling
- [x] List local models
- [x] Show model info
- [x] Add stream feature
- [x] Improve documentation
- [x] Create a model
- [x] Copy a model
- [ ] Delete a model
- [ ] Push a model
- [ ] Pull a model
- [ ] Generate embeddings
- [ ] Add missing examples in README.md