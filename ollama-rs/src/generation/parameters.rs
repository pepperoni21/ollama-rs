use schemars::{generate::SchemaSettings, Schema};
pub use schemars::{schema_for, JsonSchema};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

/// The format to return a response in
#[derive(Debug, Clone, PartialEq)]
pub enum FormatType {
    Json,

    /// Requires Ollama 0.5.0 or greater.
    StructuredJson(Box<JsonStructure>),
}

impl Serialize for FormatType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            FormatType::Json => serializer.serialize_str("json"),
            FormatType::StructuredJson(s) => s.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for FormatType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FormatTypeVisitor;

        impl<'de> Visitor<'de> for FormatTypeVisitor {
            type Value = FormatType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(r#"either the string "json" or a JSON schema object"#)
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if value == "json" {
                    Ok(FormatType::Json)
                } else {
                    Err(E::invalid_value(
                        serde::de::Unexpected::Str(value),
                        &"\"json\"",
                    ))
                }
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(&value)
            }

            fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let schema =
                    JsonStructure::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
                Ok(FormatType::StructuredJson(Box::new(schema)))
            }
        }

        deserializer.deserialize_any(FormatTypeVisitor)
    }
}

/// Represents a serialized JSON schema. You can create this by converting
/// a JsonSchema:
/// ```rust
/// let json_schema = schema_for!(Output);
/// let serialized: SerializedJsonSchema = json_schema.into();
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonStructure {
    #[serde(flatten)]
    schema: Schema,
}

impl JsonStructure {
    pub fn new<T: JsonSchema>() -> Self {
        // Need to do this because Ollama doesn't support $refs (references in the schema)
        // So we have to explicitly turn them off
        let mut settings = SchemaSettings::draft07();
        settings.inline_subschemas = true;
        let generator = settings.into_generator();
        let schema = generator.into_root_schema_for::<T>();

        Self { schema }
    }

    pub fn new_for_schema(schema: Schema) -> Self {
        Self { schema }
    }
}

/// Used to control how long a model stays loaded in memory, by default models are unloaded after 5 minutes of inactivity
#[derive(Debug, Clone, PartialEq)]
pub enum KeepAlive {
    Indefinitely,
    UnloadOnCompletion,
    Until { time: u64, unit: TimeUnit },
}

impl Serialize for KeepAlive {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            KeepAlive::Indefinitely => serializer.serialize_i8(-1),
            KeepAlive::UnloadOnCompletion => serializer.serialize_i8(0),
            KeepAlive::Until { time, unit } => {
                let mut s = String::new();
                s.push_str(&time.to_string());
                s.push_str(unit.to_symbol());
                serializer.serialize_str(&s)
            }
        }
    }
}

impl<'de> Deserialize<'de> for KeepAlive {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KeepAliveVisitor;

        impl<'de> Visitor<'de> for KeepAliveVisitor {
            type Value = KeepAlive;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("i8 value (-1 or 0), or string like \"30s\", \"5m\", \"2h\"")
            }

            fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    -1 => Ok(KeepAlive::Indefinitely),
                    0 => Ok(KeepAlive::UnloadOnCompletion),
                    _ => Err(E::invalid_value(
                        serde::de::Unexpected::Signed(v as i64),
                        &"0 or -1",
                    )),
                }
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    -1 => Ok(KeepAlive::Indefinitely),
                    0 => Ok(KeepAlive::UnloadOnCompletion),
                    _ => Err(E::invalid_value(
                        serde::de::Unexpected::Signed(v),
                        &"0 or -1",
                    )),
                }
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v == 0 {
                    Ok(KeepAlive::UnloadOnCompletion)
                } else {
                    Err(E::invalid_value(
                        serde::de::Unexpected::Unsigned(v),
                        &"0 or -1",
                    ))
                }
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v == "-1" {
                    return Ok(KeepAlive::Indefinitely);
                }
                if v == "0" {
                    return Ok(KeepAlive::UnloadOnCompletion);
                }

                let last_char = v.chars().next_back().ok_or_else(|| {
                    E::invalid_value(
                        serde::de::Unexpected::Str(v),
                        &"a string ending with s, m, or h",
                    )
                })?;

                if !last_char.is_ascii_alphabetic() {
                    return Err(E::invalid_value(
                        serde::de::Unexpected::Str(v),
                        &"a string ending with s, m, or h",
                    ));
                }

                let (num_str, unit_str) = v.split_at(v.len() - last_char.len_utf8());
                let time = num_str.parse::<u64>().map_err(|_| {
                    E::invalid_value(
                        serde::de::Unexpected::Str(v),
                        &"valid number followed by s/m/h",
                    )
                })?;

                let unit = TimeUnit::from_symbol(unit_str).ok_or_else(|| {
                    E::invalid_value(
                        serde::de::Unexpected::Str(unit_str),
                        &"one of: s, m, h (case-sensitive)",
                    )
                })?;

                Ok(KeepAlive::Until { time, unit })
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(&v)
            }
        }

        deserializer.deserialize_any(KeepAliveVisitor)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
}

impl TimeUnit {
    pub fn to_symbol(&self) -> &'static str {
        match self {
            TimeUnit::Seconds => "s",
            TimeUnit::Minutes => "m",
            TimeUnit::Hours => "h",
        }
    }

    pub fn from_symbol(s: &str) -> Option<Self> {
        match s {
            "s" => Some(TimeUnit::Seconds),
            "m" => Some(TimeUnit::Minutes),
            "h" => Some(TimeUnit::Hours),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::generation::parameters::{FormatType, JsonStructure, KeepAlive, TimeUnit};

    #[test]
    fn serde_keep_alive_indefinitely() {
        let keep_alive = KeepAlive::Indefinitely;
        let json = serde_json::to_vec(&keep_alive).unwrap();

        let parsed_keep_alive: KeepAlive = serde_json::from_slice(&json).unwrap();

        assert_eq!(keep_alive, parsed_keep_alive);
    }

    #[test]
    fn serde_keep_alive_unload_on_completion() {
        let keep_alive = KeepAlive::UnloadOnCompletion;
        let json = serde_json::to_vec(&keep_alive).unwrap();

        let parsed_keep_alive: KeepAlive = serde_json::from_slice(&json).unwrap();

        assert_eq!(keep_alive, parsed_keep_alive);
    }

    #[test]
    fn serde_keep_alive_until() {
        let keep_alive = KeepAlive::Until {
            time: 1,
            unit: TimeUnit::Seconds,
        };
        let json = serde_json::to_vec(&keep_alive).unwrap();
        let parsed_keep_alive: KeepAlive = serde_json::from_slice(&json).unwrap();
        assert_eq!(keep_alive, parsed_keep_alive);

        let keep_alive = KeepAlive::Until {
            time: 1,
            unit: TimeUnit::Minutes,
        };
        let json = serde_json::to_vec(&keep_alive).unwrap();
        let parsed_keep_alive: KeepAlive = serde_json::from_slice(&json).unwrap();
        assert_eq!(keep_alive, parsed_keep_alive);

        let keep_alive = KeepAlive::Until {
            time: 1,
            unit: TimeUnit::Hours,
        };
        let json = serde_json::to_vec(&keep_alive).unwrap();
        let parsed_keep_alive: KeepAlive = serde_json::from_slice(&json).unwrap();
        assert_eq!(keep_alive, parsed_keep_alive);
    }

    #[test]
    fn serde_format_type_json() {
        let format_type = FormatType::Json;
        let json = serde_json::to_vec(&format_type).unwrap();
        let parsed_format_type: FormatType = serde_json::from_slice(&json).unwrap();
        assert_eq!(format_type, parsed_format_type);
    }

    #[test]
    fn serde_format_type_schema() {
        let format_type = FormatType::StructuredJson(Box::new(JsonStructure {
            schema: schemars::json_schema!({
                "type": ["object", "null"]
            }),
        }));
        let json = serde_json::to_vec(&format_type).unwrap();
        let parsed_format_type: FormatType = serde_json::from_slice(&json).unwrap();
        assert_eq!(format_type, parsed_format_type);
    }
}
