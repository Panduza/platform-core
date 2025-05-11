use super::Notification;
use panduza::InstanceState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateNotification {
    /// Instance topic
    ///
    pub topic: String,

    /// State of the instance
    ///
    pub state: InstanceState,
}

impl StateNotification {
    /// Create new object
    ///
    pub fn new(name: String, state: InstanceState) -> Self {
        Self {
            topic: name,
            state: state,
        }
    }
}

/// Implicit convertion
///
impl Into<Notification> for StateNotification {
    fn into(self) -> Notification {
        Notification::State(self)
    }
}
