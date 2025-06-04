use ollama_rs::{
    generation::{
        completion::request::GenerationRequest,
        parameters::{FormatType, JsonStructure},
    },
    models::ModelOptions,
    Ollama,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ollama = Ollama::default();
    let model = "llama3.2:latest".to_string();
    let prompt = "Tell me about the country north of the USA".to_string();
    let structure = r#"
    {
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "Output",
        "type": "object",
        "properties": {
            "country": { "type": "string" },
            "capital": { "type": "string" },
            "languages": {
                "type": "array",
                "items": { "type": "string" }
            },
            "temperature": { "type": "string", "enum": ["Warm", "Cold"] }
        },
        "required": ["country", "capital", "languages", "temperature"]
    }
    "#;
    let schema = serde_json::from_str(structure)?;

    let format = FormatType::StructuredJson(Box::new(JsonStructure::new_for_schema(schema)));
    dbg!(&format);
    let res = ollama
        .generate(
            GenerationRequest::new(model, prompt)
                .format(format)
                .options(ModelOptions::default().temperature(0.0)),
        )
        .await?;

    // Output {
    //     country: "Canada",
    //     capital: "Ottawa",
    //     languages: [
    //         "English",
    //         "French",
    //     ],
    //     temperature: Cold,
    // }
    dbg!(&res.response);

    Ok(())
}
