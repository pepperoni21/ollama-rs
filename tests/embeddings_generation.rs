use ollama_rs::{generation::embeddings::request::GenerateEmbeddingsRequest, Ollama};

#[tokio::test]
async fn test_embeddings_generation() {
    let ollama = Ollama::default();

    let res = ollama
        .generate_embeddings(GenerateEmbeddingsRequest::new(
            "llama2:latest".to_string(),
            "Why is the sky blue".into(),
        ))
        .await
        .unwrap();

    dbg!(res);
}

#[tokio::test]
async fn test_batch_embeddings_generation() {
    let ollama = Ollama::default();

    let res = ollama
        .generate_embeddings(GenerateEmbeddingsRequest::new(
            "llama2:latest".to_string(),
            vec!["Why is the sky blue?", "Why is the sky red?"].into(),
        ))
        .await
        .unwrap();

    dbg!(res);
}
