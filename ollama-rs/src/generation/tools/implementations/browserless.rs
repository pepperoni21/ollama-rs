use reqwest::Client;
use schemars::JsonSchema;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::env;
use text_splitter::TextSplitter;

use serde_json::json;
use std::error::Error;

use crate::generation::tools::Tool;

#[derive(Deserialize, JsonSchema)]
pub struct Params {
    #[schemars(description = "The URL of the website to scrape")]
    website: String,
}

pub struct Browserless {}
//Add headless utilties
impl Tool for Browserless {
    type Params = Params;

    fn name() -> &'static str {
        "browserless_web_scraper"
    }

    fn description() -> &'static str {
        "Scrapes text content from websites and splits it into manageable chunks."
    }

    async fn call(&mut self, params: Self::Params) -> Result<String, Box<dyn Error + Sync + Send>> {
        let website = params.website;
        let browserless_token =
            env::var("BROWSERLESS_TOKEN").expect("BROWSERLESS_TOKEN must be set");
        let url = format!("http://0.0.0.0:3000/content?token={browserless_token}");
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
