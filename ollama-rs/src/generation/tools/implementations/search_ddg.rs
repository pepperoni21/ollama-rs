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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
        self.parse(&body)
    }
    fn parse(&self, html: &str) -> Result<Vec<SearchResult>, Box<dyn Error + Send + Sync>> {
        Html::parse_document(html)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_example() {
        let raw = r#"<div class="result results_links results_links_deep web-result ">
                          <div class="links_main links_deep result__body"> <!-- This is the visible part -->

                              <h2 class="result__title">
                                <a rel="nofollow" class="result__a" href="https://speed.cloudflare.com/">Internet Speed Test - Measure Network Performance | Cloudflare</a>
                              </h2>




                              <div class="result__extras">
                                <div class="result__extras__url">
                                  <span class="result__icon">
                                    <a rel="nofollow" href="https://speed.cloudflare.com/">
                                      <img class="result__icon__img" width="16" height="16" alt="" src="//external-content.duckduckgo.com/ip3/speed.cloudflare.com.ico" name="i15">
                                    </a>
                                  </span>
                                  <a class="result__url" href="https://speed.cloudflare.com/">
                                    speed.cloudflare.com
                                  </a>

                                </div>
                              </div>




                                <a class="result__snippet" href="https://speed.cloudflare.com/"><b>Test</b> your Internet connection and network performance with Cloudflare's global edge network. See your download, upload, latency, jitter, packet loss and network quality score.</a>



                            <div class="clear"></div>
                          </div>
                        </div>"#;
        let ddg = DDGSearcher::default();
        let x = &ddg.parse(raw).unwrap()[0];
        assert_eq!(
            x,
            &SearchResult {
                link: "speed.cloudflare.com".into(),
                title: "Internet Speed Test - Measure Network Performance | Cloudflare".into(),
                snippet: "Test your Internet connection and network performance with Cloudflare's global edge network. See your download, upload, latency, jitter, packet loss and network quality score.".into()
            }
        );
    }
}
