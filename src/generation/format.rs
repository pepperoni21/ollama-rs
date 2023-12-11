use serde::{Deserialize, Serialize};

/// The format to return a response in. Currently the only accepted value is `json`
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FormatType {
    Json,
}
