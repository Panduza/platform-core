use crate::{AttributeBuilder, ClassBuilder, Logger, TaskResult};
use async_trait::async_trait;
use std::future::Future;

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

    /// Spawn a new async task inside this instance
    ///
    async fn spawn<N: Send + Into<String>, F>(&mut self, name: N, future: F)
    where
        F: Future<Output = TaskResult> + Send + 'static;
}
