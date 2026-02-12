use ollama_rs::generation::parameters::{FormatType, KeepAlive, ThinkType, TimeUnit};

#[test]
fn serde_keep_alive_indefinitely() {
    let keep_alive = KeepAlive::Indefinitely;
    let json = serde_json::to_vec(&keep_alive).unwrap();

    let parsed_keep_alive: KeepAlive = serde_json::from_slice(&json).unwrap();

    assert_eq!(keep_alive, parsed_keep_alive);
}

#[test]
fn serde_keep_alive_unload_on_completion() {
    let keep_alive = KeepAlive::UnloadOnCompletion;
    let json = serde_json::to_vec(&keep_alive).unwrap();

    let parsed_keep_alive: KeepAlive = serde_json::from_slice(&json).unwrap();

    assert_eq!(keep_alive, parsed_keep_alive);
}

#[test]
fn serde_keep_alive_until() {
    let keep_alive = KeepAlive::Until {
        time: 1,
        unit: TimeUnit::Seconds,
    };
    let json = serde_json::to_vec(&keep_alive).unwrap();
    let parsed_keep_alive: KeepAlive = serde_json::from_slice(&json).unwrap();
    assert_eq!(keep_alive, parsed_keep_alive);

    let keep_alive = KeepAlive::Until {
        time: 1,
        unit: TimeUnit::Minutes,
    };
    let json = serde_json::to_vec(&keep_alive).unwrap();
    let parsed_keep_alive: KeepAlive = serde_json::from_slice(&json).unwrap();
    assert_eq!(keep_alive, parsed_keep_alive);

    let keep_alive = KeepAlive::Until {
        time: 1,
        unit: TimeUnit::Hours,
    };
    let json = serde_json::to_vec(&keep_alive).unwrap();
    let parsed_keep_alive: KeepAlive = serde_json::from_slice(&json).unwrap();
    assert_eq!(keep_alive, parsed_keep_alive);
}

#[test]
fn serde_format_type_json() {
    let format_type = FormatType::Json;
    let json = serde_json::to_vec(&format_type).unwrap();
    let parsed_format_type: FormatType = serde_json::from_slice(&json).unwrap();
    assert_eq!(format_type, parsed_format_type);
}

#[test]
fn serde_think_type_booleans() {
    let think_true = ThinkType::True;
    let json = serde_json::to_vec(&think_true).unwrap();
    assert_eq!(json, b"true");
    let parsed: ThinkType = serde_json::from_slice(&json).unwrap();
    assert_eq!(parsed, ThinkType::True);

    let think_false = ThinkType::False;
    let json = serde_json::to_vec(&think_false).unwrap();
    assert_eq!(json, b"false");
    let parsed: ThinkType = serde_json::from_slice(&json).unwrap();
    assert_eq!(parsed, ThinkType::False);
}

#[test]
fn serde_think_type_strings() {
    for (variant, expected) in [
        (ThinkType::Low, "\"low\""),
        (ThinkType::Medium, "\"medium\""),
        (ThinkType::High, "\"high\""),
    ] {
        let json = serde_json::to_vec(&variant).unwrap();
        assert_eq!(String::from_utf8(json.clone()).unwrap(), expected);
        let parsed: ThinkType = serde_json::from_slice(&json).unwrap();
        assert_eq!(parsed, variant);
    }
}

#[test]
fn think_type_from_bool() {
    assert_eq!(ThinkType::from(true), ThinkType::True);
    assert_eq!(ThinkType::from(false), ThinkType::False);
}
