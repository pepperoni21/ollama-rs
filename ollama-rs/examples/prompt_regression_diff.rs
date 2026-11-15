// Prompt regression diff: ask the same question under two prompt variants and
// measure exactly what changed in the answers with llm-diff — word-level ops,
// a similarity ratio, and a structural JSON diff when both answers parse as
// JSON. Run this before shipping a prompt change: not "the answer feels
// different", but "87% similar, these words moved".
//
//     cargo run --example prompt_regression_diff [model]
//
// Defaults to llama2:latest on a local Ollama server.

use llm_diff::{json_diff, TextDiff};
use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};

const MODEL: &str = "llama2:latest";

const QUESTION: &str = "What is the population of Paris, and what is it best known for?";

// Baseline vs. candidate: the change under test is the added style constraint.
const PROMPT_A: &str = "Answer in one short paragraph.";
const PROMPT_B: &str = "Answer in one short paragraph of plain text. Do not use lists.";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = std::env::args().nth(1).unwrap_or_else(|| MODEL.to_string());
    let ollama = Ollama::default();

    let answer_a = ollama
        .generate(GenerationRequest::new(
            model.clone(),
            format!("{PROMPT_A}\n\n{QUESTION}"),
        ))
        .await?
        .response;
    let answer_b = ollama
        .generate(GenerationRequest::new(
            model,
            format!("{PROMPT_B}\n\n{QUESTION}"),
        ))
        .await?
        .response;

    let diff = TextDiff::compute_words(&answer_a, &answer_b);
    println!("similarity: {:.0}%", diff.ratio * 100.0);
    if diff.is_identical() {
        println!("answers are identical — the prompt change is a no-op");
        return Ok(());
    }
    println!("changed words:");
    for op in diff.ops.iter().filter(|o| o.kind() != "=") {
        println!("  {}{}", op.kind(), op.text());
    }

    // When both answers are JSON, a text diff overstates the change (one moved
    // element marks the whole array changed); the structural diff reports the
    // JSON Pointers that actually moved.
    if let Ok(ops) = json_diff(&answer_a, &answer_b) {
        println!("structural changes: {}", ops.len());
        for op in &ops {
            println!("  {op:?}");
        }
    }
    Ok(())
}
