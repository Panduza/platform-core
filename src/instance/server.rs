pub mod boolean;
pub mod bytes;
pub mod generic;
pub mod json;
pub mod notification;
pub mod number;
pub mod status;
pub mod string;

/// The standard object attribute server
///
pub mod std_obj;
pub use std_obj::StdObjAttributeServer;

/// The attribute manages a RO stream of data
///
pub mod ro_stream;
pub use ro_stream::RoStreamAttributeServer;

use panduza::attribute::CallbackId;

/// Type alias for asynchronous callback function with generic type T
pub type CallbackFn<T> =
    Box<dyn Fn(T) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync>;

/// Type alias for condition function that filters events with generic type T
pub type ConditionFn<T> = Box<dyn Fn(&T) -> bool + Send + Sync>;

/// Asynchronous callback entry containing the callback and optional condition
pub struct CallbackEntry<T> {
    pub callback: CallbackFn<T>,
    pub condition: Option<ConditionFn<T>>,
}

// Export the async generic attribute for easier access
pub use generic::GenericAttributeServer;

impl<T> std::fmt::Debug for CallbackEntry<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallbackEntry")
            .field("callback", &"<async callback function>")
            .field(
                "condition",
                &if self.condition.is_some() {
                    "<condition function>"
                } else {
                    "None"
                },
            )
            .finish()
    }
}
