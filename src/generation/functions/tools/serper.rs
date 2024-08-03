use crate::generation::functions::tools::Tool;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::error::Error;

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

#[async_trait]
impl Tool for SerperSearchTool {
    fn name(&self) -> String {
        "google_search_tool".to_string()
    }

    fn description(&self) -> String {
        "Conducts a web search using a specified search type and returns the results.".to_string()
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                },
                "lang": {
                    "type": "string",
                    "description": "The language for the search"
                },
                "n_results": {
                    "type": "integer",
                    "description": "The number of results to return"
                }
            },
            "required": ["query"]
        })
    }
    /*
                "search_type": {
                    "type": "string",
                    "description": "The search type (search, scholar, or news)"
                }
    */

    async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
        let query = input["query"].as_str().ok_or("Query is required")?;
        let stype = input["search_type"].as_str().unwrap_or("search");
        let lang = input["lang"].as_str().unwrap_or("en");
        let n_result = input["n_results"].as_u64().unwrap_or(5);

        assert!(
            ["search", "scholar", "news"].contains(&stype),
            "Invalid search type"
        );

        let url = format!("https://google.serper.dev/{}", stype);
        let gl = if lang != "en" { lang } else { "us" };
        let n_results = std::cmp::min(n_result, 10);
        let mut payload = json!({
            "q": query,
            "gl": gl,
            "hl": lang,
            "page": 1,
            "num": n_results
        });

        if stype == "scholar" {
            payload.as_object_mut().unwrap().remove("num");
        }

        let client = Client::new();
        let api_key = env::var("SERPER_API_KEY").expect("SERPER_API_KEY must be set");
        let response = client
            .post(&url)
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
        let formatted_results = match stype {
            "search" => results
                .iter()
                .take(n_results as usize)
                .map(|r| SearchResult::from_result_data(r).to_formatted_string())
                .collect::<Vec<String>>(),
            "scholar" => results
                .iter()
                .take(n_results as usize)
                .map(|r| ScholarResult::from_result_data(r).to_formatted_string())
                .collect::<Vec<String>>(),
            "news" => results
                .iter()
                .take(n_results as usize)
                .map(|r| NewsResult::from_result_data(r).to_formatted_string())
                .collect::<Vec<String>>(),
            _ => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Invalid search type",
                )))
            }
        };

        Ok(formatted_results.join("\n"))
    }
}
