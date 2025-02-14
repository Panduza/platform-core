use super::Notification;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Notification for attribute or class deletion
///
pub struct EnablementNotification {
    /// Attribute or Class topic
    ///
    pub topic: String,

    /// true => enable, else disable
    ///
    pub enabled: bool,
}

impl EnablementNotification {
    /// Create new object
    ///
    pub fn new<A: Into<String>>(name: A, enabled: bool) -> Self {
        Self {
            topic: name.into(),
            enabled: enabled,
        }
    }
}

/// Implicit convertion
///
impl Into<Notification> for EnablementNotification {
    fn into(self) -> Notification {
        Notification::Enablement(self)
    }
}
