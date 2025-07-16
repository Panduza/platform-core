use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use super::Notification;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeMode {
    #[serde(rename = "RO")]
    ReadOnly,
    #[serde(rename = "WO")]
    WriteOnly,
    #[serde(rename = "RW")]
    ReadWrite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeNotification {
    name: String,
    pub typee: String,
    pub mode: AttributeMode,
    info: Option<String>,
    settings: Option<JsonValue>,
}

impl AttributeNotification {
    ///
    ///
    ///
    pub fn new<N: Into<String>, T: Into<String>>(
        name: N,
        typee: T,
        mode: AttributeMode,
        info: Option<String>,
        settings: Option<JsonValue>,
    ) -> Self {
        Self {
            name: name.into(),
            typee: typee.into(),
            mode,
            info: info,
            settings: settings,
        }
    }

    ///
    /// Topic getter
    ///
    pub fn topic(&self) -> String {
        self.name.clone()
    }

    ///
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn mode(&self) -> &AttributeMode {
        &self.mode
    }

    pub fn mode_into_string(&self) -> String {
        match &self.mode {
            AttributeMode::ReadOnly => "RO".to_string(),
            AttributeMode::WriteOnly => "WO".to_string(),
            AttributeMode::ReadWrite => "RW".to_string(),
        }
    }

    pub fn info(&self) -> &Option<String> {
        &self.info
    }

    pub fn settings(&self) -> &Option<JsonValue> {
        &self.settings
    }
}

/// Implicit convertion
///
impl Into<Notification> for AttributeNotification {
    fn into(self) -> Notification {
        Notification::Attribute(self)
    }
}
