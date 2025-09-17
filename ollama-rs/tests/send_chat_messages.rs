use std::sync::{Arc, Mutex};

use base64::Engine;
use futures_util::TryStreamExt;
use ollama_rs::{
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage},
        images::Image,
    },
    Ollama,
};

#[allow(dead_code)]
const PROMPT: &str = "Why is the sky blue?";

#[tokio::test]
async fn test_send_chat_messages_stream() {
    let ollama = Ollama::default();

    let messages = vec![ChatMessage::user(PROMPT.to_string())];
    let mut res = ollama
        .send_chat_messages_stream(ChatMessageRequest::new(
            "llama2:latest".to_string(),
            messages,
        ))
        .await
        .unwrap();

    let mut done = false;
    while let Some(res) = res.try_next().await.unwrap() {
        dbg!(&res);
        if res.done {
            done = true;
            break;
        }
    }

    assert!(done);
}

#[tokio::test]
async fn test_send_chat_messages() {
    let ollama = Ollama::default();

    let messages = vec![ChatMessage::user(PROMPT.to_string())];
    let res = ollama
        .send_chat_messages(ChatMessageRequest::new(
            "llama2:latest".to_string(),
            messages,
        ))
        .await
        .unwrap();
    dbg!(&res);

    assert!(res.done);
}

#[tokio::test]
async fn test_send_chat_messages_with_history_stream() {
    let ollama = Ollama::default();
    let history = Arc::new(Mutex::new(vec![]));

    let messages = vec![ChatMessage::user(PROMPT.to_string())];

    let mut done = false;

    let mut res = ollama
        .send_chat_messages_with_history_stream(
            history.clone(),
            ChatMessageRequest::new("llama2:latest".to_string(), messages),
        )
        .await
        .unwrap();

    while let Some(res) = res.try_next().await.unwrap() {
        if res.done {
            done = true;
            break;
        }
    }

    assert!(done);
    // Should have user's message as well as AI's response
    dbg!(&history.lock().unwrap());
    assert_eq!(history.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn test_send_chat_messages_with_history() {
    let ollama = Ollama::default();
    let mut history = vec![];
    let second_message = vec![ChatMessage::user("Second message".to_string())];

    let messages = vec![ChatMessage::user(PROMPT.to_string())];
    let res = ollama
        .send_chat_messages_with_history(
            &mut history,
            ChatMessageRequest::new("llama2:latest".to_string(), messages.clone()),
        )
        .await
        .unwrap();

    dbg!(&res);
    assert!(res.done);
    // Should have user's message as well as AI's response
    assert_eq!(history.len(), 2);

    let res = ollama
        .send_chat_messages_with_history(
            &mut history,
            ChatMessageRequest::new("llama2:latest".to_string(), second_message.clone()),
        )
        .await
        .unwrap();

    dbg!(&res);
    assert!(res.done);

    // Should now have 2 user messages as well as AI's responses
    assert_eq!(history.len(), 4);

    let second_user_message_in_history = history.get(2);

    assert!(second_user_message_in_history.is_some());
    assert_eq!(
        second_user_message_in_history.unwrap().content,
        "Second message".to_string()
    );
}

const IMAGE_URL: &str = "https://images.pexels.com/photos/1054655/pexels-photo-1054655.jpeg";

#[tokio::test]
async fn test_send_chat_messages_with_images() {
    let ollama = Ollama::default();

    let bytes = reqwest::get(IMAGE_URL)
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let base64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let image = Image::from_base64(&base64);

    let messages =
        vec![ChatMessage::user("What can we see in this image?".to_string()).add_image(image)];
    let res = ollama
        .send_chat_messages(ChatMessageRequest::new(
            "llava:latest".to_string(),
            messages,
        ))
        .await
        .unwrap();
    dbg!(&res);

    assert!(res.done);
}
