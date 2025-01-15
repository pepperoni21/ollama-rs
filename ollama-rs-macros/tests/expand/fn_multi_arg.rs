#[macro_use]
extern crate ollama_rs_macros;

/// Say something
///
/// * greeting - The phrase to use for greeting
/// * name - Whom to say hello to
#[function]
async fn hello_world(greeting: String, name: String) -> Result<String, Box<dyn std::error::Error>> {

    Ok(format!("{} {}", greeting, name))
}
