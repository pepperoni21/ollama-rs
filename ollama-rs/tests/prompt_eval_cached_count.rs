use ollama_rs::generation::{chat::ChatMessageResponse, completion::GenerationResponse};

const FINAL_METRICS: &str = r#"
    "total_duration": 100,
    "load_duration": 10,
    "prompt_eval_count": 26,
    "prompt_eval_duration": 20,
    "eval_count": 5,
    "eval_duration": 70
"#;

#[test]
fn chat_final_data_reads_prompt_eval_cached_count() {
    let json = format!(
        r#"{{"model": "dummy", "created_at": "2026-11-26T00:00:00Z",
            "message": {{"role": "assistant", "content": "hi"}},
            "done": true, "prompt_eval_cached_count": 24, {FINAL_METRICS}}}"#
    );

    let response: ChatMessageResponse = serde_json::from_str(&json).unwrap();
    let final_data = response.final_data.expect("final data");

    assert_eq!(final_data.prompt_eval_count, 26);
    assert_eq!(final_data.prompt_eval_cached_count, Some(24));
}

#[test]
fn chat_final_data_without_prompt_eval_cached_count() {
    let json = format!(
        r#"{{"model": "dummy", "created_at": "2026-11-26T00:00:00Z",
            "message": {{"role": "assistant", "content": "hi"}},
            "done": true, {FINAL_METRICS}}}"#
    );

    let response: ChatMessageResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(response.final_data.unwrap().prompt_eval_cached_count, None);
}

#[test]
fn generation_response_reads_prompt_eval_cached_count() {
    let json = format!(
        r#"{{"model": "dummy", "created_at": "2026-11-26T00:00:00Z",
            "response": "hi", "done": true, "prompt_eval_cached_count": 24, {FINAL_METRICS}}}"#
    );

    let response: GenerationResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(response.prompt_eval_count, Some(26));
    assert_eq!(response.prompt_eval_cached_count, Some(24));
}
