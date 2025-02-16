// use super::element::Element;
use super::{class_builder::ClassBuilder, Container};
use crate::{Error, Instance, Logger, TaskResult};
use async_trait::async_trait;
use futures::lock::Mutex;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone)]
///
///
///
pub struct Class {
    /// Class Logger
    ///
    logger: Logger,

    ///
    ///
    instance: Instance,

    ///
    ///
    topic: String,

    ///
    ///
    enabled: Arc<AtomicBool>,
    // / Sub elements
    // /
    // sub_elements: Arc<Mutex<Vec<Element>>>,
}

impl Class {
    ///
    ///
    pub fn new(builder: &ClassBuilder) -> Self {
        Class {
            logger: builder.instance.logger.new_for_class(&builder.topic),
            instance: builder.instance.clone(),
            topic: builder.topic.clone(),
            enabled: Arc::new(AtomicBool::new(true)),
            // sub_elements: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // /// Clone as an element object
    // ///
    // pub fn clone_as_element(&self) -> Element {
    //     Element::Class(self.clone())
    // }

    // /// Append a new sub element
    // ///
    // pub async fn push_sub_element(&mut self, element: Element) {
    //     self.sub_elements.lock().await.push(element);
    // }

    pub async fn change_enablement(&mut self, enabled: bool) -> Result<(), Error> {
        //
        // Flag local variable
        self.enabled.store(enabled, Ordering::Relaxed);

        //
        // Also change sub elements
        // let mut lock = self.sub_elements.lock().await;
        // for i in 0..lock.len() {
        //     if let Some(obj) = lock.get_mut(i) {
        //         obj.change_enablement(enabled).await?;
        //     }
        // }
        Ok(())
    }
}

// #[async_trait]
// impl Container for Class {
//     /// Get for the container logger
//     ///
//     fn logger(&self) -> &Logger {
//         &self.logger
//     }

//     /// Override
//     ///
//     fn create_class<N: Into<String>>(&mut self, name: N) -> ClassBuilder {
//         ClassBuilder::new(
//             Some(self.clone()),
//             self.instance.reactor.clone(),
//             self.instance.clone(),
//             format!("{}/{}", self.topic, name.into()), // take the device topic as root
//         )
//     }

//     /// Override
//     ///
//     fn create_attribute<N: Into<String>>(&mut self, name: N) -> AttributeServerBuilder {
//         self.instance
//             .reactor
//             .create_new_attribute(self.instance.r_notifier.clone())
//             .with_topic(format!("{}/{}", self.topic, name.into()))
//     }
// }
