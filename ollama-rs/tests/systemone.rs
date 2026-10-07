use std::collections::BTreeMap;

use base64::Engine;
use ollama_rs::generation::images::Image;
use ollama_rs::generation::systemone::{
    request::{Question, State, SystemOneRequest},
    SystemOneAnswer, SystemOneResponse,
};
use ollama_rs::Ollama;
use serde_json::json;

const ELEPHANT_URL: &str = "https://images.pexels.com/photos/1054655/pexels-photo-1054655.jpeg";
const TOMATO_URL: &str = "https://images.pexels.com/photos/39795138/pexels-photo-39795138.jpeg";

#[tokio::test]
async fn test_system_one_choice() {
    let ollama = Ollama::default();
    let request = SystemOneRequest::new(
        "nimble",
        "Our checkout has returned 500 errors since 9am.",
        BTreeMap::from([(
            "label".to_string(),
            Question::Choice {
                instructions: json!("Which label fits this ticket?"),
                criteria: BTreeMap::from([
                    ("billing".to_string(), json!("Payments and refunds")),
                    ("bug".to_string(), json!("Software errors")),
                    ("account".to_string(), json!("Login and account access")),
                ]),
            },
        )]),
    );

    let response = ollama.system_one(request).await.unwrap();

    assert_eq!(response.model, "nimble");
    match response.answers.get("label") {
        Some(SystemOneAnswer::Choice {
            choice,
            probabilities,
            confidence,
        }) => {
            assert_eq!(choice, "bug");
            assert_eq!(probabilities.len(), 3);
            assert!((0.0..=1.0).contains(confidence));
        }
        answer => panic!("expected a choice answer, got {answer:?}"),
    }
    assert!(response.usage.input_tokens > 0);
}

#[tokio::test]
async fn test_system_one_multiple_questions() {
    let ollama = Ollama::default();
    let request = SystemOneRequest::new(
        "nimble",
        State::object(json!({
            "ticket": "I was charged twice. Please refund the extra payment."
        })),
        BTreeMap::from([
            (
                "refund".to_string(),
                Question::Noul {
                    instructions: json!("Is the customer requesting a refund?"),
                    criteria: Some(BTreeMap::from([
                        ("false".to_string(), "No refund is requested".to_string()),
                        (
                            "true".to_string(),
                            "The customer requests a refund".to_string(),
                        ),
                    ])),
                },
            ),
            (
                "urgency".to_string(),
                Question::Score {
                    instructions: json!("How urgently does this ticket need a response?"),
                    criteria: vec![
                        "Routine: no time pressure".to_string(),
                        "Soon: a customer is inconvenienced".to_string(),
                        "Immediate: a critical service is unavailable".to_string(),
                    ],
                },
            ),
            (
                "refund_follow_up".to_string(),
                Question::Noul {
                    instructions: json!("Does the customer mention an extra payment?"),
                    criteria: None,
                },
            ),
            (
                "priority".to_string(),
                Question::Score {
                    instructions: json!("How severe is this payment issue?"),
                    criteria: vec![
                        "Minor: easy to resolve".to_string(),
                        "Moderate: affects one customer".to_string(),
                        "Major: requires immediate attention".to_string(),
                    ],
                },
            ),
            (
                "category".to_string(),
                Question::Choice {
                    instructions: json!("Which category fits this ticket?"),
                    criteria: BTreeMap::from([
                        ("billing".to_string(), json!("Payments and refunds")),
                        ("account".to_string(), json!("Account access")),
                    ]),
                },
            ),
            (
                "request_type".to_string(),
                Question::Choice {
                    instructions: json!("What kind of request is this?"),
                    criteria: BTreeMap::from([
                        ("refund".to_string(), json!("A refund request")),
                        ("question".to_string(), json!("A general question")),
                    ]),
                },
            ),
        ]),
    );

    let response = ollama.system_one(request).await.unwrap();

    assert_noul_answer(&response, "refund");
    assert_noul_answer(&response, "refund_follow_up");
    assert_score_answer(&response, "urgency", 0.0, 2.0);
    assert_score_answer(&response, "priority", 0.0, 2.0);
    assert_choice_answer(&response, "category", 2);
    assert_choice_answer(&response, "request_type", 2);
}

#[tokio::test]
async fn test_system_one_keep_alive_and_array_state() {
    let ollama = Ollama::default();
    let request = SystemOneRequest::new(
        "nimble",
        State::array([
            "Customer reports a duplicate payment.",
            "Customer asks for the extra payment to be refunded.",
        ]),
        BTreeMap::from([(
            "refund".to_string(),
            Question::Noul {
                instructions: json!("Is the customer requesting a refund?"),
                criteria: None,
            },
        )]),
    )
    .keep_alive(ollama_rs::generation::parameters::KeepAlive::Until {
        time: 5,
        unit: ollama_rs::generation::parameters::TimeUnit::Minutes,
    });

    let response = ollama.system_one(request).await.unwrap();

    assert_noul_answer(&response, "refund");
}

#[tokio::test]
async fn test_system_one_with_image() {
    let ollama = Ollama::default();
    let elephant_bytes = reqwest::get(ELEPHANT_URL)
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let elephant_base64 = base64::engine::general_purpose::STANDARD.encode(&elephant_bytes);
    let tomato_bytes = reqwest::get(TOMATO_URL)
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let tomato_base64 = base64::engine::general_purpose::STANDARD.encode(&tomato_bytes);

    let request = SystemOneRequest::new(
        "clef-flash",
        "What can we see in these images?",
        BTreeMap::from([(
            "subject".to_string(),
            Question::Choice {
                instructions: json!("Which subjects are shown across the images?"),
                criteria: BTreeMap::from([
                    ("elephant".to_string(), json!("An elephant")),
                    ("tomato".to_string(), json!("A tomato")),
                    (
                        "elephant_and_tomato".to_string(),
                        json!("An elephant and a tomato"),
                    ),
                    ("other".to_string(), json!("Something else")),
                ]),
            },
        )]),
    )
    .images(vec![
        Image::from_base64(&elephant_base64),
        Image::from_base64(&tomato_base64),
    ]);

    let response = ollama.system_one(request).await.unwrap();

    assert_choice_answer(&response, "subject", 4);
    match response.answers.get("subject") {
        Some(SystemOneAnswer::Choice { choice, .. }) => {
            assert_eq!(choice, "elephant_and_tomato");
        }
        answer => panic!("expected a choice answer for subject, got {answer:?}"),
    }
}

fn assert_noul_answer(response: &SystemOneResponse, name: &str) {
    match response.answers.get(name) {
        Some(SystemOneAnswer::Noul { noul }) => {
            assert!((0.0..=1.0).contains(noul));
        }
        answer => panic!("expected a noul answer for {name}, got {answer:?}"),
    }
}

fn assert_choice_answer(response: &SystemOneResponse, name: &str, expected_options: usize) {
    match response.answers.get(name) {
        Some(SystemOneAnswer::Choice {
            probabilities,
            confidence,
            ..
        }) => {
            assert_eq!(probabilities.len(), expected_options);
            assert!((0.0..=1.0).contains(confidence));
        }
        answer => panic!("expected a choice answer for {name}, got {answer:?}"),
    }
}

fn assert_score_answer(response: &SystemOneResponse, name: &str, min: f64, max: f64) {
    match response.answers.get(name) {
        Some(SystemOneAnswer::Score {
            score,
            probabilities,
            confidence,
            ..
        }) => {
            assert!((*score >= min) && (*score <= max));
            assert!(!probabilities.is_empty());
            assert!((0.0..=1.0).contains(confidence));
        }
        answer => panic!("expected a score answer for {name}, got {answer:?}"),
    }
}
