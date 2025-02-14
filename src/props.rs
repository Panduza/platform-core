use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value as JsonValue};
use std::collections::HashMap;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
///
/// Type of a Prop (match json types)
///
pub enum PropType {
    Bool,
    Number,
    String,
    Array,
    #[default]
    Object,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
///
///
///
pub struct Prop {
    ///
    ///
    ///
    pub description: String,

    ///
    ///
    ///
    pub r#type: PropType,

    ///
    ///
    ///
    pub default: JsonValue,
}

impl Prop {
    ///
    ///
    ///
    pub fn new<T: Into<String>>(description: T, r#type: PropType, default: JsonValue) -> Self {
        Self {
            description: description.into(),
            r#type: r#type,
            default: default,
        }
    }
}

#[derive(Default, Debug, Clone)]
///
/// Represent a group of Prop
///
pub struct Props {
    ///
    ///
    ///
    entries: HashMap<String, Prop>,
}

///
///
///
impl Serialize for Props {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.entries.serialize(serializer)
    }
}

///
/// See Serialize
///
impl<'de> Deserialize<'de> for Props {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            JsonValue::Object(map) => {
                let mut res = HashMap::<String, Prop>::new();
                for (key, entry) in map {
                    res.insert(
                        key,
                        serde_json::from_value(entry).map_err(|e| {
                            D::Error::custom(format!(" Error parsing Props map {:?}", e))
                        })?,
                    );
                }
                Ok(Self { entries: res })
            }
            _ => Err(D::Error::custom("Expected an object for Props")),
        }
    }
}

impl Props {
    ///
    ///
    ///
    pub fn add_entry<A: Into<String>, T: Into<String>>(
        &mut self,
        name: A,
        description: T,
        r#type: PropType,
        default: JsonValue,
    ) {
        self.entries
            .insert(name.into(), Prop::new(description, r#type, default));
    }

    ///
    ///
    ///
    pub fn add_string_prop<A: Into<String>, B: Into<String>, C: Into<String>>(
        &mut self,
        name: A,
        description: B,
        default: C,
    ) {
        self.add_entry(
            name,
            description,
            PropType::String,
            JsonValue::String(default.into()),
        );
    }

    ///
    ///
    ///
    pub fn add_number_prop<A: Into<String>, B: Into<String>, C: Into<f64>>(
        &mut self,
        name: A,
        description: B,
        default: C,
    ) {
        self.add_entry(
            name,
            description,
            PropType::Number,
            serde_json::json!(default.into()),
        );
    }

    ///
    ///
    ///
    pub fn add_bool_prop<A: Into<String>, B: Into<String>>(
        &mut self,
        name: A,
        description: B,
        default: bool,
    ) {
        self.add_entry(name, description, PropType::Bool, JsonValue::Bool(default));
    }
}

impl From<Map<String, JsonValue>> for Props {
    fn from(source: Map<String, JsonValue>) -> Self {
        let mut res = HashMap::<String, Prop>::new();
        for (key, entry) in source {
            res.insert(key, serde_json::from_value(entry).unwrap());
        }
        Self { entries: res }
    }
}
