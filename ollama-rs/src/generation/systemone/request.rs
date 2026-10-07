use std::collections::BTreeMap;

use serde::{Serialize, Serializer};
use serde_json::Value;

use crate::generation::{images::Image, parameters::KeepAlive};

#[derive(Debug, Clone)]
pub enum State {
    /// A single message, article, or passage.
    Text(String),
    /// An ordered sequence of text values (e.g. a list of messages).
    TextList(Vec<String>),
    /// A structured JSON value (e.g. records or application state).
    Object(Value),
}

impl State {
    /// A plain string state.
    pub fn text(text: impl Into<String>) -> Self {
        State::Text(text.into())
    }

    /// An ordered list of text values (e.g. a sequence of messages).
    pub fn texts<I, S>(list: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        State::TextList(list.into_iter().map(Into::into).collect())
    }

    /// A structured JSON object state.
    ///
    /// Accepts anything serializable, e.g. `serde_json::json!({...})` or a
    /// struct deriving `Serialize`.
    pub fn object(value: impl Serialize) -> Self {
        State::Object(serde_json::to_value(value).expect("state must be serializable"))
    }

    /// A structured JSON array state.
    pub fn array(value: impl Serialize) -> Self {
        Self::object(value)
    }
}

impl Serialize for State {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Text(value) => value.serialize(serializer),
            Self::TextList(value) => value.serialize(serializer),
            Self::Object(value) => value.serialize(serializer),
        }
    }
}

impl From<String> for State {
    fn from(text: String) -> Self {
        State::Text(text)
    }
}

impl From<&str> for State {
    fn from(text: &str) -> Self {
        State::Text(text.to_string())
    }
}

impl From<Vec<String>> for State {
    fn from(list: Vec<String>) -> Self {
        State::TextList(list)
    }
}

impl From<Value> for State {
    fn from(value: Value) -> Self {
        State::Object(value)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Question {
    Choice {
        instructions: Value,
        criteria: BTreeMap<String, Value>,
    },
    Noul {
        instructions: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        criteria: Option<BTreeMap<String, String>>,
    },
    Score {
        instructions: Value,
        criteria: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemOneRequest {
    pub model: String,
    pub state: State,
    pub questions: BTreeMap<String, Question>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<Image>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<KeepAlive>,
}

impl SystemOneRequest {
    pub fn new(
        model: impl Into<String>,
        state: impl Into<State>,
        questions: BTreeMap<String, Question>,
    ) -> Self {
        Self {
            model: model.into(),
            state: state.into(),
            questions,
            images: Vec::new(),
            keep_alive: None,
        }
    }

    pub fn images(mut self, images: Vec<Image>) -> Self {
        self.images = images;
        self
    }

    pub fn add_image(mut self, image: Image) -> Self {
        self.images.push(image);
        self
    }

    pub fn keep_alive(mut self, keep_alive: KeepAlive) -> Self {
        self.keep_alive = Some(keep_alive);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_system_one_request() {
        let mut questions = BTreeMap::new();
        questions.insert(
            "label".to_string(),
            Question::Choice {
                instructions: json!("Which label fits this ticket?"),
                criteria: BTreeMap::from([
                    ("bug".to_string(), json!("Software errors")),
                    ("billing".to_string(), Value::Null),
                ]),
            },
        );
        questions.insert(
            "urgent".to_string(),
            Question::Noul {
                instructions: json!("Is this urgent?"),
                criteria: None,
            },
        );

        let request = SystemOneRequest::new("nimble", "An outage", questions);
        let value = serde_json::to_value(request).unwrap();

        assert_eq!(value["model"], "nimble");
        assert_eq!(value["state"], "An outage");
        assert_eq!(value["questions"]["label"]["type"], "choice");
        assert_eq!(value["questions"]["urgent"]["type"], "noul");
        assert!(value["questions"]["urgent"].get("criteria").is_none());
    }
}
