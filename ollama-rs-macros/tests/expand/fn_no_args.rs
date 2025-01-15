#[macro_use]
extern crate ollama_rs_macros;

/// Say hello
#[function]
async fn hello_world() -> Result<String, Box<dyn std::error::Error>> {

    Ok("Hello".to_string())
}
