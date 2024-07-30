use reqwest::Client;
use scraper::{Html, Selector};
use std::env;
use text_splitter::TextSplitter;

use crate::generation::functions::tools::Tool;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::error::Error;

pub struct Browserless {}
//Add headless utilties
#[async_trait]
impl Tool for Browserless {
    fn name(&self) -> String {
        "browserless_web_scraper".to_string()
    }

    fn description(&self) -> String {
        "Scrapes text content from websites and splits it into manageable chunks.".to_string()
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "website": {
                    "type": "string",
                    "description": "The URL of the website to scrape"
                }
            },
            "required": ["website"]
        })
    }

    async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
        let website = input["website"].as_str().ok_or("Website URL is required")?;
        let browserless_token =
            env::var("BROWSERLESS_TOKEN").expect("BROWSERLESS_TOKEN must be set");
        let url = format!("http://0.0.0.0:3000/content?token={}", browserless_token);
        let payload = json!({
            "url": website
        });
        let client = Client::new();
        let response = client
            .post(&url)
            .header("cache-control", "no-cache")
            .header("content-type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let response_text = response.text().await?;
        let document = Html::parse_document(&response_text);
        let selector = Selector::parse("p, h1, h2, h3, h4, h5, h6").unwrap();
        let elements: Vec<String> = document
            .select(&selector)
            .map(|el| el.text().collect::<String>())
            .collect();
        let body = elements.join(" ");

        let splitter = TextSplitter::new(1000);
        let chunks = splitter.chunks(&body);
        let sentences: Vec<String> = chunks.map(|s| s.to_string()).collect();
        let sentences = sentences.join("\n \n");
        Ok(sentences)
    }
}
