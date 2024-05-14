use async_trait::async_trait;
use serde_json::{json, Value};
use std::error::Error;
use crate::generation::functions::tools::Tool;

pub struct WeatherTool;

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> String {
        "WeatherTool".to_string()
    }

    fn description(&self) -> String {
        "Get the current weather in a given location.".to_string()
    }

    async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
        let location = input.as_str().ok_or("Input should be a string")?;
        let unit = "fahrenheit";  // Default unit
        let result = if location.to_lowercase().contains("tokyo") {
            json!({"location": "Tokyo", "temperature": "10", "unit": unit})
        } else if location.to_lowercase().contains("san francisco") {
            json!({"location": "San Francisco", "temperature": "72", "unit": unit})
        } else if location.to_lowercase().contains("paris") {
            json!({"location": "Paris", "temperature": "22", "unit": unit})
        } else {
            json!({"location": location, "temperature": "unknown"})
        };

        Ok(result.to_string())
    }
}
