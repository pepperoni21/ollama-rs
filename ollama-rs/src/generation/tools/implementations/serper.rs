use reqwest::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::error::Error;

use crate::generation::tools::Tool;

#[derive(Deserialize, JsonSchema, PartialEq, Eq, Default)]
enum SearchType {
    #[default]
    Search,
    Scholar,
    News,
}

impl SearchType {
    fn name(&self) -> &'static str {
        match self {
            SearchType::Search => "search",
            SearchType::Scholar => "scholar",
            SearchType::News => "news",
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
            title: result_data
                .get("title")
                .unwrap_or(&Value::String("none".to_string()))
                .as_str()
                .unwrap()
                .to_string(),
            link: result_data
                .get("link")
                .unwrap_or(&Value::String("none".to_string()))
                .as_str()
                .unwrap()
                .to_string(),
            snippet: result_data
                .get("snippet")
                .unwrap_or(&Value::String("none".to_string()))
                .as_str()
                .unwrap()
                .to_string(),
            date: result_data
                .get("date")
                .unwrap_or(&Value::String("none".to_string()))
                .as_str()
                .unwrap()
                .to_string(),
            position: result_data
                .get("position")
                .unwrap_or(&Value::Number(serde_json::Number::from(-1)))
                .as_i64()
                .unwrap() as i32,
        }
    }

    pub fn to_formatted_string(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}",
            self.title, self.link, self.snippet, self.date, self.position
        )
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScholarResult {
    title: String,
    link: String,
    publication_info: String,
    snippet: String,
    year: i32,
    cited_by: i32,
}

impl ScholarResult {
    pub fn from_result_data(result_data: &Value) -> Self {
        Self {
            title: result_data
                .get("title")
                .unwrap_or(&Value::String("none".to_string()))
                .as_str()
                .unwrap()
                .to_string(),
            link: result_data
                .get("link")
                .unwrap_or(&Value::String("none".to_string()))
                .as_str()
                .unwrap()
                .to_string(),
            publication_info: result_data
                .get("publicationInfo")
                .unwrap_or(&Value::String("none".to_string()))
                .as_str()
                .unwrap()
                .to_string(),
            snippet: result_data
                .get("snippet")
                .unwrap_or(&Value::String("none".to_string()))
                .as_str()
                .unwrap()
                .to_string(),
            year: result_data
                .get("year")
                .unwrap_or(&Value::Number(serde_json::Number::from(-1)))
                .as_i64()
                .unwrap() as i32,
            cited_by: result_data
                .get("citedBy")
                .unwrap_or(&Value::Number(serde_json::Number::from(-1)))
                .as_i64()
                .unwrap() as i32,
        }
    }

    pub fn to_formatted_string(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            self.title, self.link, self.publication_info, self.snippet, self.year, self.cited_by
        )
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewsResult {
    title: String,
    link: String,
    snippet: String,
    date: String,
    source: String,
    image_url: String,
    position: i32, // -1 indicates missing position
}

impl NewsResult {
    pub fn from_result_data(result_data: &Value) -> Self {
        Self {
            title: result_data
                .get("title")
                .unwrap_or(&Value::String("none".to_string()))
                .as_str()
                .unwrap()
                .to_string(),
            link: result_data
                .get("link")
                .unwrap_or(&Value::String("none".to_string()))
                .as_str()
                .unwrap()
                .to_string(),
            snippet: result_data
                .get("snippet")
                .unwrap_or(&Value::String("none".to_string()))
                .as_str()
                .unwrap()
                .to_string(),
            date: result_data
                .get("date")
                .unwrap_or(&Value::String("none".to_string()))
                .as_str()
                .unwrap()
                .to_string(),
            source: result_data
                .get("source")
                .unwrap_or(&Value::String("none".to_string()))
                .as_str()
                .unwrap()
                .to_string(),
            image_url: result_data
                .get("imageUrl")
                .unwrap_or(&Value::String("none".to_string()))
                .as_str()
                .unwrap()
                .to_string(),
            position: result_data
                .get("position")
                .unwrap_or(&Value::Number(serde_json::Number::from(-1)))
                .as_i64()
                .unwrap() as i32,
        }
    }

    pub fn to_formatted_string(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}",
            self.title,
            self.link,
            self.snippet,
            self.date,
            self.source,
            self.image_url,
            self.position
        )
    }
}

pub struct SerperSearchTool;

impl Tool for SerperSearchTool {
    type Params = Params;

    fn name() -> &'static str {
        "google_search_tool"
    }

    fn description() -> &'static str {
        "Conducts a web search using a specified search type and returns the results."
    }

    async fn call(&mut self, params: Params) -> Result<String, Box<dyn Error + Sync + Send>> {
        let lang = params.lang.as_deref().unwrap_or("en");
        let url = format!("https://google.serper.dev/{}", params.search_type.name());
        let gl = if lang != "en" { lang } else { "us" };
        let n_results = params.n_results.unwrap_or(5).min(10);
        let mut payload = json!({
            "q": params.query,
            "gl": gl,
            "hl": lang,
            "page": 1,
            "num": n_results
        });

        if params.search_type == SearchType::Scholar {
            payload.as_object_mut().unwrap().remove("num");
        }

        let client = Client::new();
        let api_key = env::var("SERPER_API_KEY").expect("SERPER_API_KEY must be set");
        let response = client
            .post(url)
            .header("X-API-KEY", api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?
            .json::<Value>()
            .await?;

        let results = response["organic"]
            .as_array()
            .ok_or("Invalid response format")?;
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
