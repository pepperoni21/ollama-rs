use ollama_rs::{
    generation::{
        completion::{request::GenerationRequest, GenerationResponse},
        parameters::FormatType,
    },
    Ollama,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value; // Import Value

#[derive(Deserialize, Serialize, JsonSchema)]
struct Country {
    name: String,
    capital: String,
    languages: Vec<String>,
}

fn clean_schema(mut schema_value: Value) -> Value {
    if let Some(obj) = schema_value.as_object_mut() {
        obj.remove("$schema");
        obj.remove("title");
    }
    schema_value
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ollama = Ollama::default();
    let model = "llama3.2".to_string(); // Or llama3.2 if available
    let prompt = "Tell me about Canada.".to_string();

    let schema = schemars::schema_for!(Country);
    let format_json = clean_schema(serde_json::to_value(&schema).unwrap());
    let request = GenerationRequest::new(model, prompt).format(FormatType::Json(format_json));

    let res: GenerationResponse = ollama.generate(request).await?;

    // Attempt to parse the response as JSON, pretty print if successful,
    // otherwise print the raw response string.
    match serde_json::from_str::<Value>(&res.response) {
        Ok(json_value) => {
            println!("{}", serde_json::to_string_pretty(&json_value)?);
        }
        Err(_) => {
            println!("{}", res.response);
        }
    }

    Ok(())
}
