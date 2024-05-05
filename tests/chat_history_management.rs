use ollama_rs::Ollama;

#[test]
fn test_chat_history_saved_as_should() {
    let mut ollama = Ollama::new_default_with_history(30);
    let chat_id = "default".to_string();

    ollama.add_user_response(&chat_id, "Hello".to_string());
    ollama.add_assistant_response(&chat_id, "Hi".to_string());

    ollama.add_user_response(&chat_id, "Tell me 'hi' again".to_string());
    ollama.add_assistant_response(&chat_id, "Hi again".to_string());

    let history = ollama.get_messages_history(&chat_id).unwrap();

    assert_eq!(history.len(), 4);

    let last = history.last();
    assert!(last.is_some());
    assert_eq!(last.unwrap().content, "Hi again".to_string());
}
