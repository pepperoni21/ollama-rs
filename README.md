# Ollama-rs

### A simple and easy-to-use library for interacting with the Ollama API.

This library was created following the [Ollama API](https://github.com/jmorganca/ollama/blob/main/docs/api.md) documentation.

## Table of Contents

- [Installation](#installation)
- [Initialization](#initialization)
- [Usage](#usage)
  - [Completion Generation](#completion-generation)
  - [Completion Generation (Streaming)](#completion-generation-streaming)
  - [Completion Generation (With Options)](#completion-generation-with-options)
  - [Chat Mode](#chat-mode)
  - [List Local Models](#list-local-models)
  - [Show Model Information](#show-model-information)
  - [Create a Model](#create-a-model)
  - [Create a Model (Streaming)](#create-a-model-streaming)
  - [Copy a Model](#copy-a-model)
  - [Delete a Model](#delete-a-model)
  - [Generate Embeddings](#generate-embeddings)
  - [Generate Embeddings (Batch)](#generate-embeddings-batch)
  - [Make a Function Call](#make-a-function-call)
  - [Create a custom tool](#create-a-custom-tool)
  - [Completion Generation (With Thinking)](#completion-generation-with-thinking)

## Installation

### Add ollama-rs to your Cargo.toml

```toml
[dependencies]
ollama-rs = "0.3.2"
```

If you absolutely want the latest version, you can use the `master` branch by adding the following to your `Cargo.toml` file:

```toml
[dependencies]
ollama-rs = { git = "https://github.com/pepperoni21/ollama-rs.git", branch = "master" }
```

*Note that the `master` branch may not be stable and may contain breaking changes.*

## Initialization

### Initialize Ollama

```rust
use ollama_rs::Ollama;

// By default, it will connect to localhost:11434
let ollama = Ollama::default();

// For custom values:
let ollama = Ollama::new("http://localhost".to_string(), 11434);
```

## Usage

Feel free to check the [Chatbot example](https://github.com/pepperoni21/ollama-rs/blob/0.3.2/ollama-rs/examples/basic_chatbot.rs) that shows how to use the library to create a simple chatbot in less than 50 lines of code. You can also check some [other examples](https://github.com/pepperoni21/ollama-rs/tree/0.3.2/ollama-rs/examples).

_These examples use poor error handling for simplicity, but you should handle errors properly in your code._

### Completion Generation

```rust
use ollama_rs::generation::completion::GenerationRequest;

let model = "llama2:latest".to_string();
let prompt = "Why is the sky blue?".to_string();

let res = ollama.generate(GenerationRequest::new(model, prompt)).await;

if let Ok(res) = res {
    println!("{}", res.response);
}
```

**OUTPUTS:** _The sky appears blue because of a phenomenon called Rayleigh scattering..._

### Completion Generation (Streaming)

_Requires the `stream` feature._

```rust
use ollama_rs::generation::completion::GenerationRequest;
use tokio::io::{self, AsyncWriteExt};
use tokio_stream::StreamExt;

let model = "llama2:latest".to_string();
let prompt = "Why is the sky blue?".to_string();

let mut stream = ollama.generate_stream(GenerationRequest::new(model, prompt)).await.unwrap();

let mut stdout = io::stdout();
while let Some(res) = stream.next().await {
    let responses = res.unwrap();
    for resp in responses {
        stdout.write_all(resp.response.as_bytes()).await.unwrap();
        stdout.flush().await.unwrap();
    }
}
```

Same output as above but streamed.

### Completion Generation (With Options)

```rust
use ollama_rs::generation::completion::GenerationRequest;
use ollama_rs::models::ModelOptions;

let model = "llama2:latest".to_string();
let prompt = "Why is the sky blue?".to_string();

let options = ModelOptions::default()
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

### Chat Mode

_Every message sent and received will be stored in the library's history._

Example with history:

```rust
use ollama_rs::generation::chat::{ChatMessage, ChatMessageRequest};
use ollama_rs::history::ChatHistory;

let model = "llama2:latest".to_string();
let prompt = "Why is the sky blue?".to_string();
// `Vec<ChatMessage>` implements `ChatHistory`,
// but you could also implement it yourself on a custom type
let mut history = vec![];

let res = ollama
    .send_chat_messages_with_history(
        &mut history, // <- messages will be saved here
        ChatMessageRequest::new(
            model,
            vec![ChatMessage::user(prompt)], // <- You should provide only one message
        ),
    )
    .await;

if let Ok(res) = res {
    println!("{}", res.message.content);
}
```

_Check chat with history examples for [default](https://github.com/pepperoni21/ollama-rs/blob/0.3.2/ollama-rs/examples/chat_with_history.rs) and [stream](https://github.com/pepperoni21/ollama-rs/blob/0.3.2/ollama-rs/examples/chat_with_history_stream.rs)_

### List Local Models

```rust
let res = ollama.list_local_models().await.unwrap();
```

_Returns a vector of `LocalModel` structs._

### Show Model Information

```rust
let res = ollama.show_model_info("llama2:latest".to_string()).await.unwrap();
```

_Returns a `ModelInfo` struct._

### Create a Model

```rust
use ollama_rs::models::create::CreateModelRequest;

let res = ollama.create_model(CreateModelRequest::path("model".into(), "/tmp/Modelfile.example".into())).await.unwrap();
```

_Returns a `CreateModelStatus` struct representing the final status of the model creation._

### Create a Model (Streaming)

_Requires the `stream` feature._

```rust
use ollama_rs::models::create::CreateModelRequest;
use tokio_stream::StreamExt;

let mut res = ollama.create_model_stream(CreateModelRequest::path("model".into(), "/tmp/Modelfile.example".into())).await.unwrap();

while let Some(res) = res.next().await {
    let res = res.unwrap();
    // Handle the status
}
```

_Returns a `CreateModelStatusStream` that will stream every status update of the model creation._

### Copy a Model

```rust
let _ = ollama.copy_model("mario".into(), "mario_copy".into()).await.unwrap();
```

### Delete a Model

```rust
let _ = ollama.delete_model("mario_copy".into()).await.unwrap();
```

### Generate Embeddings

```rust
use ollama_rs::generation::embeddings::request::GenerateEmbeddingsRequest;

let request = GenerateEmbeddingsRequest::new("llama2:latest".to_string(), "Why is the sky blue?".into());
let res = ollama.generate_embeddings(request).await.unwrap();
```

### Generate Embeddings (Batch)

```rust
use ollama_rs::generation::embeddings::request::GenerateEmbeddingsRequest;

let request = GenerateEmbeddingsRequest::new("llama2:latest".to_string(), vec!["Why is the sky blue?", "Why is the sky red?"].into());
let res = ollama.generate_embeddings(request).await.unwrap();
```

_Returns a `GenerateEmbeddingsResponse` struct containing the embeddings (a vector of floats)._

### Make a Function Call

```rust
use ollama_rs::coordinator::Coordinator;
use ollama_rs::generation::chat::{ChatMessage, ChatMessageRequest};
use ollama_rs::generation::tools::implementations::{DDGSearcher, Scraper, Calculator};
use ollama_rs::models::ModelOptions;

let mut history = vec![];

let mut coordinator = Coordinator::new(ollama, "qwen2.5:32b".to_string(), history)
    .options(ModelOptions::default().num_ctx(16384))
    .add_tool(DDGSearcher::new())
    .add_tool(Scraper {})
    .add_tool(Calculator {});

let resp = coordinator
    .chat(vec![ChatMessage::user("What is the current oil price?")])
    .await.unwrap();

println!("{}", resp.message.content);
```

_Uses the given tools (such as searching the web) to find an answer, feeds that answer back into the LLM, and returns a `ChatMessageResponse` with the answer to the question._

### Create a custom tool

The `function` macro simplifies the creation of custom tools. Below is an example of a tool that retrieves the current weather for a specified city:

```rust
/// Retrieve the weather for a specified city.
///
/// * city - The city for which to get the weather.
#[ollama_rs::function]
async fn get_weather(city: String) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    let url = format!("https://wttr.in/{city}?format=%C+%t");
    let response = reqwest::get(&url).await?.text().await?;
    Ok(response)
}
```

To create a custom tool, define a function that returns a `Result<String, Box<dyn std::error::Error + Sync + Send>>` and annotate it with the `function` macro. This function will be automatically converted into a tool that can be used with the `Coordinator`, just like any other tool.

Ensure that the doc comment above the function clearly describes the tool's purpose and its parameters. This information will be provided to the LLM to help it understand how to use the tool.

For a more detailed example, see the [function call example](https://github.com/pepperoni21/ollama-rs/blob/0.3.2/ollama-rs/examples/function_call.rs).

### Completion Generation (With Thinking)

```rust
let model = "qwen3:latest".to_string();
let prompt = "Why is the sky blue?".to_string();

let res = ollama.generate(GenerationRequest::new(model, prompt).think(true)).await;

if let Ok(res) = res {
    println!("{}", res.response);
}
```
