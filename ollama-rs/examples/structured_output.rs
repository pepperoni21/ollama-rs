use ollama_rs::{
    generation::{
        completion::request::GenerationRequest,
        parameters::{FormatType, JsonSchema, JsonStructure},
    },
    models::ModelOptions,
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
    let model = "llama3.2:latest".to_string();
    let prompt = "Tell me about the country north of the USA".to_string();

    let format = FormatType::StructuredJson(Box::new(JsonStructure::new::<Output>()));
    dbg!(&format);
    let res = ollama
        .generate(
            GenerationRequest::new(model, prompt)
                .format(format)
                .options(ModelOptions::default().temperature(0.0)),
        )
        .await?;

    dbg!(&res.response);
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
