use std::path::PathBuf;

use ollama_rs::{coordinator::Coordinator, generation::chat::ChatMessage, Ollama};

/// Get the CPU temperature in Celsius.
#[ollama_rs::function]
async fn get_cpu_temperature() -> Result<String, Box<dyn std::error::Error>> {
    Ok("42.7".to_string())
}

/// Get the available space in bytes for a given path.
///
/// * path - Path to check available space for.
#[ollama_rs::function]
async fn get_available_space(path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    Ok(fs2::available_space(path).map_or_else(
        // Note: this will let LLM handle the error. Return `Err` if you want to bubble it up.
        |err| format!("failed to get available space: {err}"),
        |space| space.to_string(),
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ollama = Ollama::default();
    let history = vec![];
    let tools = (get_cpu_temperature, get_available_space);
    let mut coordinator =
        Coordinator::new_with_tools(ollama, "llama3.2".to_string(), history, tools);

    let user_messages = vec![
        "What's the CPU temperature?",
        "What's the available space in the root directory?",
    ];

    for user_message in user_messages {
        println!("User: {user_message}");

        let user_message = ChatMessage::user(user_message.to_owned());
        let resp = coordinator.chat(vec![user_message]).await?;
        println!("Assistant: {}", resp.message.content);
    }

    Ok(())
}
