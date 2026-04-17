use serde_json::{json, Value};

use ollama_rs::models::ModelOptions;

fn assert_conversions(name: &str, opts: ModelOptions, value: Value) {
    let encoded = serde_json::to_value(&opts);
    assert!(
        encoded.is_ok(),
        "{name}: ModelOptions → json::Value: {:?}",
        encoded.err()
    );
    assert_eq!(
        encoded.unwrap(),
        value,
        "{name}: ModelOptions → json::Value"
    );

    let parsed = serde_json::from_value::<ModelOptions>(value);
    assert!(
        parsed.is_ok(),
        "{name}: json::Value → ModelOptions: {:?}",
        parsed.err()
    );
    assert_eq!(
        parsed.as_ref().unwrap(),
        &opts,
        "{name}: json::Value → ModelOptions"
    );
}

#[test]
fn model_options_json_data_driven() {
    for (name, opts, value) in [
        (
            "extra fields flattened",
            ModelOptions::default()
                .extra("penalize_newline", true)
                .extra("custom_float", 1.5),
            json!({
                "custom_float": 1.5,
                "penalize_newline": true,
            }),
        ),
        (
            "known options merged with extra",
            ModelOptions::default()
                .top_k(40)
                .extra("presence_penalty", 0.25),
            json!({
                "presence_penalty": 0.25,
                "top_k": 40,
            }),
        ),
        (
            "default omits empty extra map",
            ModelOptions::default().seed(42),
            json!({ "seed": 42 }),
        ),
        (
            "extras inserts multiple entries",
            ModelOptions::default().extras([("x", json!(1)), ("y", json!("two"))]),
            json!({
                "x": 1,
                "y": "two",
            }),
        ),
        (
            "unknown keys round-trip via extra",
            ModelOptions::default()
                .seed(7)
                .extra("penalize_newline", true)
                .extra("nested", json!({ "a": 1 })),
            json!({
                "nested": { "a": 1 },
                "penalize_newline": true,
                "seed": 7,
            }),
        ),
    ] {
        assert_conversions(name, opts, value);
    }
}
