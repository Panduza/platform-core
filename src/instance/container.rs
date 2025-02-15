use super::class_builder::ClassBuilder;
use crate::Logger;
use async_trait::async_trait;
use panduza::AttributeBuilder;

#[async_trait]
/// Common interface shared between Instance and Class
///
/// It allows parent container abstraction, Instance can be seen as the top level class
///
pub trait Container: Clone {
    /// Get for the container logger
    ///
    fn logger(&self) -> &Logger;

    /// Create a new interface from this device
    ///
    fn create_class<N: Into<String>>(&mut self, name: N) -> ClassBuilder;

    /// Device can directly create some attribute on its root
    ///
    fn create_attribute<N: Into<String>>(&mut self, name: N) -> AttributeBuilder;
}
