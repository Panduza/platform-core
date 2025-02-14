use crate::{Error, Props};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]

///
///
///
pub struct Product {
    ///
    ///
    ///
    pub description: String,

    ///
    ///
    ///
    pub props: Props,
}

#[derive(Default, Debug, Clone)]
/// # Store
///
/// Contains information about drivers that can be produced by a factory
///
/// ## Json Structure
///
/// {
///     "dref" {
///         "description": "....",
///         "props": {}
///     }
/// }
///
pub struct Store {
    ///
    ///
    ///
    pub products: HashMap<String, Product>,
}

///
///
///
impl Serialize for Store {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.products.serialize(serializer)
    }
}

///
/// See Serialize
///
impl<'de> Deserialize<'de> for Store {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            JsonValue::Object(map) => {
                let mut res = HashMap::<String, Product>::new();
                for (key, entry) in map {
                    res.insert(
                        key,
                        serde_json::from_value(entry).map_err(|e| {
                            serde::de::Error::custom(format!(" Error parsing Product map {:?}", e))
                        })?,
                    );
                }
                Ok(Self { products: res })
            }
            _ => Err(serde::de::Error::custom("Expected an object for Store")),
        }
    }
}

impl Store {
    ///
    /// Check if the store contains the given product ref
    ///
    pub fn contains(&self, r#ref: &String) -> bool {
        self.products.contains_key(r#ref)
    }

    ///
    /// Extend the current store by copying an other
    ///
    pub fn extend_by_copy(&mut self, other: &Store) {
        self.products.extend(other.products.clone());
    }

    ///
    ///
    ///
    pub fn into_json_value(&self) -> Result<JsonValue, Error> {
        serde_json::to_value(&self.products).map_err(|e| Error::InternalLogic(format!("{:?}", e)))
    }
}
