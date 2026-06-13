use ollama_rs::{
    generation::chat::{request::ChatMessageRequest, ChatMessage, MessageRole},
    Ollama,
};

#[tokio::test]
async fn test_chat_history_accumulated() {
    let ollama = Ollama::default();

    let mut history = vec![];

    let model = std::env::var("OLLAMA_RS_TEST_MODEL").unwrap_or("gemma3:4b-cloud".to_owned());

    assert!(ollama
        .send_chat_messages_with_history(
            &mut history,
            ChatMessageRequest::new(
                // FIXME: eugh
                model.clone(),
                vec![ChatMessage::new(
                    MessageRole::User,
                    "Why is the sky blue?".into(),
                )],
            ),
        )
        .await
        .is_ok());

    assert!(ollama
        .send_chat_messages_with_history(
            &mut history,
            ChatMessageRequest::new(
                model,
                vec![ChatMessage::new(
                    MessageRole::User,
                    "But, why is the sky blue?".into()
                )]
            ),
        )
        .await
        .is_ok());

    assert_eq!(history.len(), 4)
}
