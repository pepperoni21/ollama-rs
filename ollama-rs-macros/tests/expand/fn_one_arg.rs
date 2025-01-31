#[macro_use]
extern crate ollama_rs_macros;

/// Say hello
///
/// * name - Whom to say hello to
#[function]
async fn hello_world(name: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    Ok(format!("Hello {}", name))
}
