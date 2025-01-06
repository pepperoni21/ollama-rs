use regex::Regex;
use crate::{error::OllamaError, Ollama};

fn extract_models_from_html(data: String) -> Vec<String> {
  let mut models: Vec<String> = Vec::new();

  // This is the regular expression to
  // capture models names in HTML content.
  let re = Regex::new(r"<span x-test-search-response-title>(.*?)</span>").unwrap();

  for cap in re.captures_iter(data.as_str()) {
    models.push(cap[1].to_string());
  }

  return models
}

impl Ollama {
  pub async fn list_online_models(&self, model_type: Option<&str>) -> crate::error::Result<Vec<String>> {
    let mut online_ollama_url = "https://ollama.com/search".to_string();

    match model_type.as_deref() {
      Some("vision") | Some("tools") | Some("embedding") => {
        online_ollama_url = format!("{}?c={}", online_ollama_url, model_type.unwrap());
      }
      None => {}
      _ => return Err(crate::error::OllamaError::Other("Please select a valid type.".to_string())),
    }

    let builder = self.reqwest_client.get(online_ollama_url);

    #[cfg(feature = "headers")]
    let builder = builder.headers(self.request_headers.clone());

    let res = builder.send().await?;

    if !res.status().is_success() {
      return Err(OllamaError::Other(res.text().await?));
    }

    let response = res.bytes().await?;
    let data = String::from_utf8(response.to_vec()).unwrap();
    let models = extract_models_from_html(data);

    // let models: Vec<String> = vec!["abc".to_string(), "def".to_string(), "ghi".to_string()];
    Ok(models)
  }
}
