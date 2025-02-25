#[cfg(feature = "modelfile")]
use modelfile::modelfile::{Instruction, Parameter};

#[tokio::test]
async fn test_show_model_info() {
    let ollama = ollama_rs::Ollama::default();

    let model_info = ollama
        .show_model_info("llama2:latest".to_string())
        .await
        .unwrap();

    dbg!(model_info);
}

#[cfg(feature = "modelfile")]
#[tokio::test]
async fn test_model_info_modelfile_param_stop() {
    let ollama = ollama_rs::Ollama::default();

    let model_info = ollama
        .show_model_info("llama2:latest".to_string())
        .await
        .unwrap();

    let stop = model_info
        .modelfile
        .instructions()
        .find_map(|i| match i {
            Instruction::Parameter(Parameter::Stop(s)) => Some(s),
            _ => None,
        })
        .unwrap();

    assert_eq!(stop, "[INST]");
}
