use ollama_rs::{
    generation::chat::ChatMessage,
    generation::functions::tools::{DDGSearcher, Scraper},
    generation::functions::{FunctionCallRequest, NousFunctionCall},
    Ollama,
};
use std::sync::Arc;
use tokio::io::{stdout, AsyncWriteExt};

#[tokio::test]
async fn test_send_function_call() {
    let mut ollama = Ollama::new_default_with_history(30);
    let scraper_tool = Arc::new(Scraper {});
    let ddg_search_tool = Arc::new(DDGSearcher::new());

    let query = "".to_string();
    let user_message = ChatMessage::user(query.to_string());

    let parser = Arc::new(NousFunctionCall {});
    let result = ollama
        .send_function_call(
            FunctionCallRequest::new(
                "adrienbrault/nous-hermes2pro:Q8_0".to_string(),
                vec![scraper_tool.clone(), ddg_search_tool.clone()],
                vec![user_message.clone()],
            ),
            parser.clone(),
        )
        .await.unwrap();

    dbg!(&result);

}
