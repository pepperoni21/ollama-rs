use futures_util::TryStreamExt;
use ollama_rs::Ollama;

#[tokio::test]
async fn test_pull_model() {
    let ollama = Ollama::default();

    let mut res = ollama
        .pull_model_stream("llama2:latest".into(), false)
        .await
        .unwrap();

    while let Some(res) = res.try_next().await.unwrap() {
        println!("{res:?}");
    }
}
