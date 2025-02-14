use super::Notification;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Notification about interface creation
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassNotification {
    /// Class topic
    ///
    pub topic: String,

    /// Interfaces tags
    ///
    pub tags: Vec<String>,
}

impl ClassNotification {
    ///
    ///
    pub fn new<N: Into<String>>(topic: N, tags: Vec<String>) -> Self {
        Self {
            topic: topic.into(),
            tags,
        }
    }

    /// Topic getter
    ///
    pub fn topic(&self) -> String {
        self.topic.clone()
    }

    pub fn into_json_value(&self) -> serde_json::Value {
        //
        // let mut children = serde_json::Map::new();
        // for e in &self.elements {
        //     children.insert(e.name().clone(), e.into_json_value());
        // }

        return json!({
            "tags": self.tags,
            // "children": children
        });
    }
}

/// Implicit convertion
///
impl Into<Notification> for ClassNotification {
    fn into(self) -> Notification {
        Notification::Class(self)
    }
}
