use ollama_rs::{
    generation::{
        completion::request::GenerationRequest,
        options::GenerationOptions,
        parameters::{schema_for, FormatType, JsonSchema, JsonStructure},
    },
    Ollama,
};
use serde::Deserialize;

#[derive(JsonSchema, Deserialize, Debug)]
enum Temperature {
    Warm,
    Cold,
}

#[allow(dead_code)]
#[derive(JsonSchema, Deserialize, Debug)]
struct Output {
    country: String,
    capital: String,
    languages: Vec<String>,
    temperature: Temperature,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ollama = Ollama::default();
    let model = "llama3.1:latest".to_string();
    let prompt = "Tell me about the country north of the USA".to_string();

    let format = FormatType::StructuredJson(JsonStructure::new::<Output>());
    let res = ollama
        .generate(
            GenerationRequest::new(model, prompt)
                .format(format)
                .options(GenerationOptions::default().temperature(0.0)),
        )
        .await?;

    let resp: Output = serde_json::from_str(&res.response)?;

    // Output {
    //     country: "Canada",
    //     capital: "Ottawa",
    //     languages: [
    //         "English",
    //         "French",
    //     ],
    //     temperature: Cold,
    // }
    dbg!(resp);

    Ok(())
}
