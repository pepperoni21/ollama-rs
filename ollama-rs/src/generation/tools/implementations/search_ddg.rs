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
    text_selectors: [Selector; 3],
    result_selector: Selector,
}

impl Default for DDGSearcher {
    fn default() -> Self {
        Self::new()
    }
}

const TEXT_SELECTORS: [&str; 3] = [".result__a", ".result__url", ".result__snippet"];

impl DDGSearcher {
    pub fn new() -> Self {
        DDGSearcher {
            client: reqwest::Client::new(),
            base_url: "https://html.duckduckgo.com".to_string(),
            text_selectors: TEXT_SELECTORS.map(|s| Selector::parse(s).unwrap()),
            result_selector: Selector::parse(".web-result").unwrap(),
        }
    }

    pub async fn search(
        &self,
        query: &str,
    ) -> Result<Vec<SearchResult>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/html/?q={}", self.base_url, query);
        let resp = self.client.get(&url).send().await?;
        let body = resp.text().await?;
        let document = Html::parse_document(&body);

        document
            .select(&self.result_selector)
            .map(|result| {
                let [title, link, snippet] = std::array::from_fn(|i| {
                    let selector = &self.text_selectors[i];
                    let text_selector = &TEXT_SELECTORS[i];
                    result
                        .select(selector)
                        .next()
                        .ok_or_else(|| {
                            format!(
                                "couldn't find selector {text_selector} in '{}'",
                                result.html()
                            )
                        })
                        .map(|e| e.text().collect::<String>())
                });

                Ok(SearchResult {
                    title: title?,
                    link: link?.trim().to_string(),
                    snippet: snippet?,
                })
            })
            .collect()
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
