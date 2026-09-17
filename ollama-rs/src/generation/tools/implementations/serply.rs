use reqwest::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::error::Error;

use crate::generation::tools::Tool;

const API_BASE: &str = "https://api.serply.io/v1";
/// Serply returns at most 10 rows per page and ignores a larger `num`, so the
/// request is clamped rather than promising the caller more than will arrive.
const MAX_RESULTS: i32 = 10;

#[derive(Deserialize, JsonSchema, Default)]
enum SearchType {
    #[default]
    Search,
    Scholar,
    News,
}

impl SearchType {
    fn path(&self) -> &'static str {
        match self {
            SearchType::Search => "search",
            SearchType::Scholar => "scholar",
            SearchType::News => "news",
        }
    }

    /// Each vertical returns its rows under a different top-level key.
    fn results_key(&self) -> &'static str {
        match self {
            SearchType::Search => "results",
            SearchType::Scholar => "articles",
            SearchType::News => "entries",
        }
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct Params {
    #[schemars(description = "The search type")]
    #[serde(default)]
    search_type: SearchType,
    #[schemars(description = "The search query")]
    query: String,
    #[schemars(description = "The language for the search")]
    lang: Option<String>,
    #[schemars(description = "The number of results to return")]
    n_results: Option<i32>,
}

fn string_field(data: &Value, key: &str) -> String {
    data.get(key)
        .and_then(Value::as_str)
        .unwrap_or("none")
        .to_string()
}

fn nested_string_field(data: &Value, path: &[&str]) -> String {
    let mut current = data;
    for key in path {
        match current.get(key) {
            Some(value) => current = value,
            None => return "none".to_string(),
        }
    }
    current.as_str().unwrap_or("none").to_string()
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResult {
    title: String,
    link: String,
    snippet: String,
    date: String,
    position: i32, // -1 indicates missing position
}

impl SearchResult {
    pub fn from_result_data(result_data: &Value) -> Self {
        Self {
            title: string_field(result_data, "title"),
            link: string_field(result_data, "link"),
            snippet: string_field(result_data, "description"),
            date: nested_string_field(result_data, &["metadata", "published_time"]),
            position: result_data
                .get("position")
                .and_then(Value::as_i64)
                .unwrap_or(-1) as i32,
        }
    }

    pub fn to_formatted_string(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}",
            self.title, self.link, self.snippet, self.date, self.position
        )
    }
}

/// Serply reports the citation count as a display string, e.g. "Cited by 13".
fn citation_count(result_data: &Value) -> i32 {
    result_data
        .get("extras")
        .and_then(|extras| extras.get("citations"))
        .and_then(|citations| citations.get("count"))
        .and_then(Value::as_str)
        .and_then(|count| count.rsplit(' ').next())
        .and_then(|count| count.parse().ok())
        .unwrap_or(-1)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScholarResult {
    title: String,
    link: String,
    publication_info: String,
    cited_by: i32, // -1 indicates missing citation count
}

impl ScholarResult {
    pub fn from_result_data(result_data: &Value) -> Self {
        Self {
            title: string_field(result_data, "title"),
            link: string_field(result_data, "link"),
            // The scholar rows repeat this byline verbatim in `description`, so it
            // is surfaced once rather than as both a byline and a snippet.
            publication_info: nested_string_field(result_data, &["author", "names"]),
            cited_by: citation_count(result_data),
        }
    }

    pub fn to_formatted_string(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}",
            self.title, self.link, self.publication_info, self.cited_by
        )
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewsResult {
    title: String,
    link: String,
    date: String,
    source: String,
}

impl NewsResult {
    pub fn from_result_data(result_data: &Value) -> Self {
        Self {
            title: string_field(result_data, "title"),
            link: string_field(result_data, "link"),
            date: string_field(result_data, "published"),
            source: nested_string_field(result_data, &["source", "title"]),
        }
    }

    pub fn to_formatted_string(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}",
            self.title, self.link, self.date, self.source
        )
    }
}

pub struct SerplySearchTool;

impl Tool for SerplySearchTool {
    type Params = Params;

    fn name() -> &'static str {
        "serply_search_tool"
    }

    fn description() -> &'static str {
        "Conducts a Google web, scholar, or news search using a specified search type and returns the results."
    }

    async fn call(&mut self, params: Params) -> Result<String, Box<dyn Error + Sync + Send>> {
        let lang = params.lang.as_deref().unwrap_or("en");
        let n_results = params.n_results.unwrap_or(5).clamp(1, MAX_RESULTS);
        let api_key = env::var("SERPLY_API_KEY").map_err(|_| "SERPLY_API_KEY must be set")?;

        let response = Client::new()
            .get(format!("{}/{}", API_BASE, params.search_type.path()))
            .query(&[("q", params.query.as_str()), ("hl", lang)])
            .query(&[("num", n_results)])
            .header("X-Api-Key", api_key)
            .header("User-Agent", "ollama-rs")
            .send()
            .await?
            .json::<Value>()
            .await?;

        let results = response[params.search_type.results_key()]
            .as_array()
            .ok_or("Invalid response format")?;

        // The news vertical serves a fixed-size feed and ignores `num`, so the
        // count is enforced here for every vertical.
        let formatted_results = match params.search_type {
            SearchType::Search => results
                .iter()
                .take(n_results as usize)
                .map(|r| SearchResult::from_result_data(r).to_formatted_string())
                .collect::<Vec<String>>(),
            SearchType::Scholar => results
                .iter()
                .take(n_results as usize)
                .map(|r| ScholarResult::from_result_data(r).to_formatted_string())
                .collect::<Vec<String>>(),
            SearchType::News => results
                .iter()
                .take(n_results as usize)
                .map(|r| NewsResult::from_result_data(r).to_formatted_string())
                .collect::<Vec<String>>(),
        };

        Ok(formatted_results.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_a_web_result() {
        let raw = json!({
            "title": "The State of Async Rust: Runtimes",
            "link": "https://corrode.dev/blog/async/",
            "description": "Tokio stands as Rust's canonical async runtime.",
            "position": 1,
            "metadata": { "published_time": "Jul 30, 2026" }
        });

        let result = SearchResult::from_result_data(&raw);

        assert_eq!(
            result.to_formatted_string(),
            "The State of Async Rust: Runtimes\n\
             https://corrode.dev/blog/async/\n\
             Tokio stands as Rust's canonical async runtime.\n\
             Jul 30, 2026\n\
             1"
        );
    }

    #[test]
    fn parses_a_scholar_result_and_its_citation_count() {
        let raw = json!({
            "title": "Overview of embedded rust operating systems",
            "link": "https://www.mdpi.com/1424-8220/24/17/5818",
            "author": { "names": "T Vandervelden - Sensors, 2024 - mdpi.com" },
            "extras": { "citations": { "count": "Cited by 13" } }
        });

        let result = ScholarResult::from_result_data(&raw);

        assert_eq!(
            result.to_formatted_string(),
            "Overview of embedded rust operating systems\n\
             https://www.mdpi.com/1424-8220/24/17/5818\n\
             T Vandervelden - Sensors, 2024 - mdpi.com\n\
             13"
        );
    }

    #[test]
    fn parses_a_news_result() {
        let raw = json!({
            "title": "Async Programming in Rust",
            "link": "https://news.google.com/rss/articles/CBMiigFB",
            "published": "Thu, 03 Sep 2026 17:17:42 GMT",
            "source": { "title": "thenewstack.io" }
        });

        let result = NewsResult::from_result_data(&raw);

        assert_eq!(
            result.to_formatted_string(),
            "Async Programming in Rust\n\
             https://news.google.com/rss/articles/CBMiigFB\n\
             Thu, 03 Sep 2026 17:17:42 GMT\n\
             thenewstack.io"
        );
    }

    #[test]
    fn missing_fields_fall_back_instead_of_panicking() {
        let raw = json!({ "title": "Only a title" });

        let result = SearchResult::from_result_data(&raw);

        assert_eq!(
            result.to_formatted_string(),
            "Only a title\nnone\nnone\nnone\n-1"
        );
        assert_eq!(citation_count(&raw), -1);
    }
}
