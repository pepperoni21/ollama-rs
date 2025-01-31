use reqwest::Client;
use schemars::JsonSchema;
use serde::Deserialize;
use std::error::Error;

use crate::generation::tools::Tool;

#[derive(Deserialize, JsonSchema)]
pub struct Params {
    #[schemars(description = "The URL of the website to scrape")]
    website: String,
}

pub struct Scraper {}

impl Default for Scraper {
    fn default() -> Self {
        Self::new()
    }
}

impl Scraper {
    pub fn new() -> Self {
        Self {}
    }
}

impl Tool for Scraper {
    type Params = Params;

    fn name() -> &'static str {
        "website_scraper"
    }

    fn description() -> &'static str {
        "Scrapes text content from websites and splits it into manageable chunks."
    }

    async fn call(&mut self, params: Self::Params) -> Result<String, Box<dyn Error + Sync + Send>> {
        let client = Client::new();
        let response = client.get(params.website).send().await?.text().await?;

        Ok(html2md::parse_html(&response))
    }
}
