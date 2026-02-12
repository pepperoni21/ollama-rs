use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

/// Controls whether thinking/reasoning models will think before responding.
///
/// `True` and `False` serialize as booleans, while `Low`, `Medium`, and `High`
/// serialize as strings `"low"`, `"medium"`, and `"high"`.
#[derive(Debug, Clone, PartialEq)]
pub enum ThinkType {
    True,
    False,
    Low,
    Medium,
    High,
}

impl Serialize for ThinkType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ThinkType::True => serializer.serialize_bool(true),
            ThinkType::False => serializer.serialize_bool(false),
            ThinkType::Low => serializer.serialize_str("low"),
            ThinkType::Medium => serializer.serialize_str("medium"),
            ThinkType::High => serializer.serialize_str("high"),
        }
    }
}

impl<'de> Deserialize<'de> for ThinkType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ThinkTypeVisitor;

        impl<'de> Visitor<'de> for ThinkTypeVisitor {
            type Value = ThinkType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(r#"a boolean or one of "low", "medium", "high""#)
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v {
                    Ok(ThinkType::True)
                } else {
                    Ok(ThinkType::False)
                }
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    "low" => Ok(ThinkType::Low),
                    "medium" => Ok(ThinkType::Medium),
                    "high" => Ok(ThinkType::High),
                    _ => Err(E::invalid_value(
                        serde::de::Unexpected::Str(v),
                        &r#""low", "medium", or "high""#,
                    )),
                }
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(&v)
            }
        }

        deserializer.deserialize_any(ThinkTypeVisitor)
    }
}

impl From<bool> for ThinkType {
    fn from(value: bool) -> Self {
        if value {
            ThinkType::True
        } else {
            ThinkType::False
        }
    }
}
