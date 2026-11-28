use ollama_rs::{
    generation::completion::{request::GenerationRequest, GenerationResponseStream},
    Ollama,
};
use tokio_stream::StreamExt;

const PROMPT: &str = "Say hello";
const MODEL: &str = "llama2:latest";

#[tokio::test]
async fn test_generation_with_logprobs() {
    let ollama = Ollama::default();

    let res = ollama
        .generate(
            GenerationRequest::new(MODEL.to_string(), PROMPT)
                .logprobs(true)
                .top_logprobs(3),
        )
        .await
        .unwrap();

    // Verify that logprobs are returned
    assert!(
        res.logprobs.is_some(),
        "Expected logprobs to be present in response"
    );

    let logprobs = res.logprobs.unwrap();
    assert!(
        !logprobs.is_empty(),
        "Expected logprobs to contain at least one entry"
    );

    // Verify structure of logprobs data. We deliberately do not assert that
    // `token` or `bytes` are non-empty: `token` can be an empty string for
    // byte-level tokens, and `bytes` is `omitempty` in the Ollama API (so it
    // is absent when a token has no byte representation).
    for logprob_data in &logprobs {
        // logprob should be a valid float (negative or zero, since it's a log probability)
        assert!(
            logprob_data.logprob <= 0.0,
            "Expected logprob to be <= 0.0, got {}",
            logprob_data.logprob
        );
        // top_logprobs should be present since the request asked for them
        assert!(
            logprob_data.top_logprobs.is_some(),
            "Expected top_logprobs to be present"
        );
        if let Some(ref top_logprobs) = logprob_data.top_logprobs {
            assert!(
                !top_logprobs.is_empty(),
                "Expected top_logprobs to contain at least one entry"
            );
            for top_logprob_data in top_logprobs {
                assert!(
                    top_logprob_data.logprob <= 0.0,
                    "Expected logprob in top_logprobs to be <= 0.0, got {}",
                    top_logprob_data.logprob
                );
            }
        }
    }

    println!("Non-streaming test passed. Response: {}", res.response);
    println!("Number of logprobs entries: {}", logprobs.len());
}

#[tokio::test]
#[cfg(feature = "stream")]
async fn test_generation_stream_with_logprobs() {
    let ollama = Ollama::default();

    let mut res: GenerationResponseStream = ollama
        .generate_stream(
            GenerationRequest::new(MODEL.to_string(), PROMPT)
                .logprobs(true)
                .top_logprobs(3),
        )
        .await
        .unwrap();

    let mut done = false;
    let mut total_logprobs_entries = 0;
    let mut full_response = String::new();

    while let Some(res) = res.next().await {
        let res = res.unwrap();
        for ele in res {
            full_response.push_str(&ele.response);

            // Check if logprobs are present in streaming chunks
            if let Some(ref logprobs) = ele.logprobs {
                total_logprobs_entries += logprobs.len();

                // Verify structure of logprobs data in each chunk. As with the
                // non-streaming test, `token` and `bytes` are not asserted to
                // be non-empty (they are not guaranteed by the Ollama API).
                for logprob_data in logprobs {
                    assert!(
                        logprob_data.logprob <= 0.0,
                        "Expected logprob to be <= 0.0 in streaming chunk, got {}",
                        logprob_data.logprob
                    );
                }
            }

            if ele.done {
                done = true;
                break;
            }
        }
    }

    assert!(done, "Expected streaming to complete");
    assert!(
        total_logprobs_entries > 0,
        "Expected to receive logprobs entries in streaming response"
    );

    println!("Streaming test passed. Full response: {}", full_response);
    println!(
        "Total logprobs entries received: {}",
        total_logprobs_entries
    );
}
