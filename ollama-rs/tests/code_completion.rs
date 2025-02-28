use ollama_rs::{generation::completion::request::GenerationRequest, models::ModelOptions, Ollama};

const CODE_MODEL: &str = "granite-code:3b";

#[tokio::test]
async fn typical_c_code_main() {
    const C_PREFIX: &str = "int m";
    const C_SUFFIX: &str = "(int argc, char **argv)";
    const C_COMPLETION: &str = "ain";

    let options = ModelOptions::default().seed(146);
    let request =
        GenerationRequest::new_with_suffix(CODE_MODEL.into(), C_PREFIX.into(), C_SUFFIX.into())
            .options(options);

    let ollama = Ollama::default();
    let res = ollama.generate(request).await.unwrap();

    let completion = res.response;
    assert_eq!(completion, C_COMPLETION);
}
