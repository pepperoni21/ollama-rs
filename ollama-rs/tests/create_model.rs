use ollama_rs::{models::create::CreateModelRequest, Ollama};
use tokio_stream::StreamExt;

#[tokio::test]
/// This test needs a Modelfile at /tmp to work
async fn test_create_model_stream() {
    let ollama = Ollama::default();

    let request = CreateModelRequest::new("testmodel".into())
        .license("Test".into())
        .system("You're a chat bot. (very useful information)".into())
        .template("Template".into())
        .from_model("llama2:latest".into());

    let mut res = ollama.create_model_stream(request).await.unwrap();

    let mut done = false;
    while let Some(res) = res.next().await {
        match res {
            Ok(res) => {
                dbg!(&res.message);
                if res.message.eq("success") {
                    done = true;
                }
            }
            Err(e) => panic!("{e:?}"),
        }
    }

    assert!(done);
}

#[tokio::test]
/// This test needs a Modelfile at /tmp to work
async fn test_create_model() {
    let ollama = Ollama::default();

    let request = CreateModelRequest::new("testmodel".into())
        .license("Test".into())
        .system("You're a chat bot. (very useful information)".into())
        .template("Template".into())
        .from_model("llama2:latest".into());

    let res = ollama.create_model(request).await.unwrap();

    assert!(res.message.eq("success"));
}
