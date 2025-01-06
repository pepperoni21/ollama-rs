#[tokio::test]
async fn test_list_online_models() {
  let ollama = ollama_rs::Ollama::default();

  // let models = ollama.list_online_models(Some("vision")).await.unwrap();
  // let models = ollama.list_online_models(Some("tools")).await.unwrap();
  // let models = ollama.list_online_models(Some("embedding")).await.unwrap();
  let models = ollama.list_online_models(None).await.unwrap();

  dbg!(models);
}
