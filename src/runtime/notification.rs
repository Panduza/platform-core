pub mod alert;
pub mod attribute;
pub mod class;
pub mod enablement;
pub mod group;
pub mod state;

pub use alert::AlertNotification;
pub use attribute::AttributeNotification;
pub use class::ClassNotification;
pub use enablement::EnablementNotification;
pub use state::StateNotification;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Available Runtime Notification Types
///
pub enum Notification {
    /// There is a warning message coming from the instance
    ///
    Alert(AlertNotification),

    /// An instance state has changed
    ///
    State(StateNotification),

    /// A class has been created
    ///
    Class(ClassNotification),

    /// An attribute has been created
    ///
    Attribute(AttributeNotification),

    /// An attribute or a class has been enabled or disabled
    ///
    /// Deletion does not exist, once created only the instance destruction
    /// can erase the attribute or the class. Choose Enable/Disable instead.
    ///
    Enablement(EnablementNotification),
}
