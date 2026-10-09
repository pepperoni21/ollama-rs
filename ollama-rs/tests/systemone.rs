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
    let request =
        SystemOneRequest::builder("nimble", "Our checkout has returned 500 errors since 9am.")
            .question(
                "label",
                Question::choice(
                    "Which label fits this ticket?",
                    [
                        ("billing", "Payments and refunds"),
                        ("bug", "Software errors"),
                        ("account", "Login and account access"),
                    ],
                ),
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
    let request = SystemOneRequest::builder(
        "nimble",
        State::object(json!({
            "ticket": "I was charged twice. Please refund the extra payment."
        })),
    )
    .question(
        "refund",
        Question::noul_with_criteria(
            "Is the customer requesting a refund?",
            [
                ("false", "No refund is requested"),
                ("true", "The customer requests a refund"),
            ],
        ),
    )
    .question(
        "urgency",
        Question::score(
            "How urgently does this ticket need a response?",
            [
                "Routine: no time pressure",
                "Soon: a customer is inconvenienced",
                "Immediate: a critical service is unavailable",
            ],
        ),
    )
    .question(
        "refund_follow_up",
        Question::noul("Does the customer mention an extra payment?"),
    )
    .question(
        "priority",
        Question::score(
            "How severe is this payment issue?",
            [
                "Minor: easy to resolve",
                "Moderate: affects one customer",
                "Major: requires immediate attention",
            ],
        ),
    )
    .question(
        "category",
        Question::choice(
            "Which category fits this ticket?",
            [
                ("billing", "Payments and refunds"),
                ("account", "Account access"),
            ],
        ),
    )
    .question(
        "request_type",
        Question::choice(
            "What kind of request is this?",
            [
                ("refund", "A refund request"),
                ("question", "A general question"),
            ],
        ),
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
    let request = SystemOneRequest::builder(
        "nimble",
        State::array([
            "Customer reports a duplicate payment.",
            "Customer asks for the extra payment to be refunded.",
        ]),
    )
    .question(
        "refund",
        Question::noul("Is the customer requesting a refund?"),
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

    let request = SystemOneRequest::builder("clef-flash", "What can we see in these images?")
        .question(
            "subject",
            Question::choice(
                "Which subjects are shown across the images?",
                [
                    ("elephant", "An elephant"),
                    ("tomato", "A tomato"),
                    ("elephant_and_tomato", "An elephant and a tomato"),
                    ("other", "Something else"),
                ],
            ),
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
