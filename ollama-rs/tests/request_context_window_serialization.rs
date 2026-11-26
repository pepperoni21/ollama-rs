use ollama_rs::generation::{
    chat::{request::ChatMessageRequest, ChatMessage},
    completion::request::GenerationRequest,
};

#[test]
fn chat_request_serializes_truncate_and_shift() {
    let request = ChatMessageRequest::new(
        "dummy".to_string(),
        vec![ChatMessage::user("hi".to_string())],
    )
    .truncate(false)
    .shift(true);

    let value = serde_json::to_value(&request).unwrap();

    assert_eq!(value["truncate"], false);
    assert_eq!(value["shift"], true);
}

#[test]
fn generation_request_serializes_truncate_and_shift() {
    let request = GenerationRequest::new("dummy".to_string(), "Why?")
        .truncate(true)
        .shift(false);

    let value = serde_json::to_value(&request).unwrap();

    assert_eq!(value["truncate"], true);
    assert_eq!(value["shift"], false);
}

#[test]
fn requests_omit_truncate_and_shift_by_default() {
    let chat = ChatMessageRequest::new("dummy".to_string(), vec![]);
    let generation = GenerationRequest::new("dummy".to_string(), "Why?");

    for value in [
        serde_json::to_value(&chat).unwrap(),
        serde_json::to_value(&generation).unwrap(),
    ] {
        assert!(value.get("truncate").is_none());
        assert!(value.get("shift").is_none());
    }
}
