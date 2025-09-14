use ollama_rs::{
    generation::chat::{request::ChatMessageRequest, ChatMessage, MessageRole},
    Ollama,
};

#[tokio::test]
async fn test_chat_history_accumulated() {
    let ollama = Ollama::default();

    let mut history = vec![];

    assert!(ollama
        .send_chat_messages_with_history(
            &mut history,
            ChatMessageRequest::new(
                "granite-code:3b".into(),
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
                "granite-code:3b".into(),
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
