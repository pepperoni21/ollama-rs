use ollama_rs::Ollama;

#[test]
fn test_chat_history_saved_as_should() {
    let mut ollama = Ollama::new_default_with_history(30);
    let chat_id = "default".to_string();

    ollama.add_user_response(chat_id.clone(), "Hello".to_string());
    ollama.add_assistant_response(chat_id.clone(), "Hi".to_string());

    ollama.add_user_response(chat_id.clone(), "Tell me 'hi' again".to_string());
    ollama.add_assistant_response(chat_id.clone(), "Hi again".to_string());

    assert_eq!(
        ollama.get_messages_history(chat_id.clone()).unwrap().len(),
        4
    );

    let last = ollama.get_messages_history(chat_id).unwrap().last();
    assert!(last.is_some());
    assert_eq!(last.unwrap().content, "Hi again".to_string());
}
