use super::Notification;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertNotification {
    /// Class topic
    ///
    pub topic: String,

    /// Alert message
    ///
    pub message: String,
}

impl AlertNotification {
    /// Create object
    ///
    pub fn new(topic: String, message: String) -> Self {
        Self {
            topic: topic,
            message: message,
        }
    }
}

/// Implicit convertion
///
impl Into<Notification> for AlertNotification {
    fn into(self) -> Notification {
        Notification::Alert(self)
    }
}
