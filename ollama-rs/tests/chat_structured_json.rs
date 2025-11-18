use ollama_rs::{
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage},
        parameters::{FormatType, JsonSchema, JsonStructure},
    },
    models::ModelOptions,
    Ollama,
};
use serde::Deserialize;
use tokio_stream::StreamExt;

#[derive(Debug, Deserialize, JsonSchema)]
struct CountryInfo {
    country: String,
    capital: String,
}

fn structured_format() -> FormatType {
    FormatType::StructuredJson(Box::new(JsonStructure::new::<CountryInfo>()))
}

fn structured_prompt() -> Vec<ChatMessage> {
    vec![ChatMessage::user(
        "Return a JSON object with fields 'country' and 'capital' describing Canada.".to_string(),
    )]
}

#[tokio::test]
async fn chat_structured_json_single() {
    let ollama = Ollama::default();

    let request = ChatMessageRequest::new("gpt-oss:latest".to_string(), structured_prompt())
        .options(ModelOptions::default().temperature(0.0))
        .format(structured_format());

    let response = ollama
        .send_chat_messages(request)
        .await
        .expect("chat call succeeds");

    assert!(response.done);

    let parsed: CountryInfo =
        serde_json::from_str(&response.message.content).expect("response parses into CountryInfo");

    assert_eq!(parsed.country, "Canada");
    assert_eq!(parsed.capital, "Ottawa");
}

#[tokio::test]
async fn chat_structured_json_stream() {
    let ollama = Ollama::default();

    let request = ChatMessageRequest::new("gpt-oss:latest".to_string(), structured_prompt())
        .options(ModelOptions::default().temperature(0.0))
        .format(structured_format());

    let mut stream = ollama
        .send_chat_messages_stream(request)
        .await
        .expect("chat stream starts");

    let mut aggregated = String::new();
    let mut final_message = None;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.expect("chunk ok");
        aggregated.push_str(&chunk.message.content);

        if chunk.done {
            final_message = Some(chunk);
            break;
        }
    }

    let _final_message = final_message.expect("received done chunk");

    let parsed: CountryInfo =
        serde_json::from_str(&aggregated).expect("stream response parses into CountryInfo");

    assert_eq!(parsed.country, "Canada");
    assert_eq!(parsed.capital, "Ottawa");
}
