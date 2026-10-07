use ollama_rs::{
    generation::systemone::{
        request::{Question, SystemOneRequest},
        SystemOneAnswer,
    },
    Ollama,
};

const MODEL: &str = "nimble";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ollama = Ollama::default();
    let request =
        SystemOneRequest::builder(MODEL, "Our checkout has returned 500 errors since 9am.")
            .question(
                "refund",
                Question::noul("Is the customer requesting a refund?"),
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

    let response = ollama.system_one(request).await?;

    for (name, answer) in response.answers {
        print!("{name}: ");
        match answer {
            SystemOneAnswer::Choice { choice, .. } => println!("choice = {choice}"),
            SystemOneAnswer::Noul { noul } => println!("true probability = {noul}"),
            SystemOneAnswer::Score { score, .. } => println!("score = {score}"),
        }
    }

    println!(
        "Usage: {} input tokens, {} output tokens",
        response.usage.input_tokens, response.usage.output_tokens
    );

    Ok(())
}
