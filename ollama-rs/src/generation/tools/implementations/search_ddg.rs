use reqwest;

use schemars::JsonSchema;
use scraper::{Html, Selector};
use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::generation::tools::Tool;

#[derive(Deserialize, JsonSchema)]
pub struct Params {
    #[schemars(description = "The search query to send to DuckDuckGo")]
    query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    title: String,
    link: String,
    snippet: String,
}

pub struct DDGSearcher {
    pub client: reqwest::Client,
    pub base_url: String,
}

impl Default for DDGSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl DDGSearcher {
    pub fn new() -> Self {
        DDGSearcher {
            client: reqwest::Client::new(),
            base_url: "https://duckduckgo.com".to_string(),
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/html/?q={}", self.base_url, query);
        let resp = self.client.get(&url).send().await?;
        let body = resp.text().await?;
        let document = Html::parse_document(&body);

        let result_selector = Selector::parse(".web-result").unwrap();
        let result_title_selector = Selector::parse(".result__a").unwrap();
        let result_url_selector = Selector::parse(".result__url").unwrap();
        let result_snippet_selector = Selector::parse(".result__snippet").unwrap();

        let results = document
            .select(&result_selector)
            .map(|result| {
                let title = result
                    .select(&result_title_selector)
                    .next()
                    .unwrap()
                    .text()
                    .collect::<Vec<_>>()
                    .join("");
                let link = result
                    .select(&result_url_selector)
                    .next()
                    .unwrap()
                    .text()
                    .collect::<Vec<_>>()
                    .join("")
                    .trim()
                    .to_string();
                let snippet = result
                    .select(&result_snippet_selector)
                    .next()
                    .unwrap()
                    .text()
                    .collect::<Vec<_>>()
                    .join("");

                SearchResult {
                    title,
                    link,
                    //url: String::from(url.value().attr("href").unwrap()),
                    snippet,
                }
            })
            .collect::<Vec<_>>();

        Ok(results)
    }
}

impl Tool for DDGSearcher {
    type Params = Params;

    fn name() -> &'static str {
        "ddg_searcher"
    }

    fn description() -> &'static str {
        "Searches the web using DuckDuckGo's HTML interface."
    }

    async fn call(&mut self, params: Params) -> Result<String, Box<dyn Error + Sync + Send>> {
        let results = self.search(&params.query).await?;
        let results_json = serde_json::to_string(&results)?;
        Ok(results_json)
    }
}
