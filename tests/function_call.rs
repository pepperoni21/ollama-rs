// #![cfg(feature = "function-calling")]

use ollama_rs::{
    generation::chat::ChatMessage,
    generation::functions::tools::{DDGSearcher, Scraper},
    generation::functions::{FunctionCallRequest, NousFunctionCall},
    Ollama,
};
use std::sync::Arc;

#[tokio::test]
async fn test_send_function_call() {
    /// Model to be used, make sure it is tailored towards "function calling", such as:
    /// - openhermes:latest
    /// - adrienbrault/nous-hermes2pro:Q8_0
    const MODEL: &str = "adrienbrault/nous-hermes2pro:Q8_0";

    const PROMPT: &str = "";
    let user_message = ChatMessage::user(PROMPT.to_string());

    let scraper_tool = Arc::new(Scraper::new());
    let ddg_search_tool = Arc::new(DDGSearcher::new());
    let parser = Arc::new(NousFunctionCall::new());

    let ollama = Ollama::new_default_with_history(30);
    let result = ollama
        .send_function_call(
            FunctionCallRequest::new(
                MODEL.to_string(),
                vec![scraper_tool, ddg_search_tool],
                vec![user_message],
            ),
            parser,
        )
        .await
        .unwrap();

    assert!(result.done);
}
