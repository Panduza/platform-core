pub mod boolean;
pub mod bytes;
pub mod r#enum;
pub mod generic;
pub mod json;
pub mod notification_v0;
pub mod number;
pub mod sample;
pub mod si;
pub mod status_v0;
pub mod string;
pub mod trigger_v0;
pub mod vector_f32_v0;

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
