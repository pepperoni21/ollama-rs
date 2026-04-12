use ollama_rs::Ollama;
use tokio_stream::StreamExt;

#[tokio::test]
/// This test needs a local model named `test_model:latest` to work, and requires registering for ollama.ai and adding a public key first.
/// The model name should be in the form of `<username>/<model>:<tag>`.
async fn test_push_model() {
    let ollama = Ollama::default();

    let model_name = format!("{}/test_model:latest", env!("USER"));

    ollama
        .copy_model("test_model".into(), model_name.clone())
        .await
        .unwrap();

    let mut res = ollama.push_model_stream(model_name, false).await.unwrap();

    while let Some(res) = res.next().await {
        match res {
            Ok(res) => println!("{res:?}"),
            Err(e) => panic!("{e:?}"),
        }
    }
}
