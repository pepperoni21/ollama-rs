use ollama_rs::generation::chat::{ChatMessage, ChatMessageResponse, MessageRole};

#[test]
fn deserialize_tool_call_id_from_response() {
    let json = r#"{
        "model": "dummy",
        "created_at": "2026-11-26T00:00:00Z",
        "message": {
            "role": "assistant",
            "content": "",
            "tool_calls": [
                {"id": "call_1", "function": {"name": "get_weather", "arguments": {"city": "Paris"}}},
                {"function": {"name": "get_time", "arguments": {}}}
            ]
        },
        "done": false
    }"#;

    let response: ChatMessageResponse = serde_json::from_str(json).unwrap();
    let calls = &response.message.tool_calls;

    assert_eq!(calls[0].id.as_deref(), Some("call_1"));
    assert_eq!(calls[0].function.name, "get_weather");
    assert_eq!(calls[1].id, None);
}

#[test]
fn serialize_tool_message_with_name_and_call_id() {
    let message = ChatMessage::tool("18C".to_string())
        .with_tool_name("get_weather")
        .with_tool_call_id("call_1");

    let value = serde_json::to_value(&message).unwrap();

    assert_eq!(value["role"], "tool");
    assert_eq!(value["tool_name"], "get_weather");
    assert_eq!(value["tool_call_id"], "call_1");
}

#[test]
fn serialize_message_without_tool_fields_omits_them() {
    let value = serde_json::to_value(ChatMessage::user("hi".to_string())).unwrap();

    assert!(value.get("tool_name").is_none());
    assert!(value.get("tool_call_id").is_none());
}

#[test]
fn deserialize_message_without_tool_fields() {
    let message: ChatMessage =
        serde_json::from_str(r#"{"role": "assistant", "content": "hello"}"#).unwrap();

    assert_eq!(message.role, MessageRole::Assistant);
    assert_eq!(message.tool_name, None);
    assert_eq!(message.tool_call_id, None);
}
