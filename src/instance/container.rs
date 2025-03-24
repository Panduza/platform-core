use super::attribute_builder::AttributeServerBuilder;
use super::class_builder::ClassBuilder;
use crate::Logger;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Notify;

#[async_trait]
/// Common interface shared between Instance and Class
///
/// It allows parent container abstraction, Instance can be seen as the top level class
///
pub trait Container: Clone {
    /// Get for the container logger
    ///
    fn logger(&self) -> &Logger;

    /// Signal to request a new init of the attribute
    ///
    fn reset_signal(&self) -> Arc<Notify>;

    ///
    ///
    fn trigger_reset_signal(&self);

    /// Create a new interface from this device
    ///
    fn create_class<N: Into<String>>(&mut self, name: N) -> ClassBuilder;

    /// Device can directly create some attribute on its root
    ///
    fn create_attribute<N: Into<String>>(&mut self, name: N) -> AttributeServerBuilder;
}
