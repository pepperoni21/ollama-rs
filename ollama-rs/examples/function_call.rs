use std::path::PathBuf;

use ollama_rs::{coordinator::Coordinator, generation::chat::ChatMessage, Ollama};

/// Get the CPU temperature in Celsius.
#[ollama_rs::function]
async fn get_cpu_temperature() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    Ok("42.7".to_string())
}

/// Get the available space in bytes for a given path.
///
/// * path - Path to check available space for.
#[ollama_rs::function]
async fn get_available_space(
    path: PathBuf,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    Ok(fs2::available_space(path).map_or_else(
        // Note: this will let LLM handle the error. Return `Err` if you want to bubble it up.
        |err| format!("failed to get available space: {err}"),
        |space| space.to_string(),
    ))
}

/// Get the weather for a given city.
///
/// * city - City to get the weather for.
#[ollama_rs::function]
async fn get_weather(city: String) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    Ok(reqwest::get(format!("https://wttr.in/{city}?format=%C+%t"))
        .await?
        .text()
        .await?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    let ollama = Ollama::default();
    let history = vec![];
    let mut coordinator = Coordinator::new(ollama, "llama3.2".to_string(), history)
        .add_tool(get_cpu_temperature)
        .add_tool(get_available_space)
        .add_tool(get_weather);

    let user_messages = vec![
        "What's the CPU temperature?",
        "What's the available space in the root directory?",
        "What's the weather in Berlin?",
    ];

    for user_message in user_messages {
        println!("User: {user_message}");

        let user_message = ChatMessage::user(user_message.to_owned());
        let resp = coordinator.chat(vec![user_message]).await?;
        println!("Assistant: {}", resp.message.content);
    }

    Ok(())
}
