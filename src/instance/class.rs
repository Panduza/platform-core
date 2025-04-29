use super::attribute_builder::AttributeServerBuilder;
use super::{class_builder::ClassBuilder, Container};
use crate::{Error, Instance, Logger, Notification};
use async_trait::async_trait;
use panduza::task_monitor::{NamedTaskHandle, TaskHandle};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::Notify;

#[derive(Clone)]
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

    ///
    ///
    notification_channel: Sender<Notification>,
    // / Sub elements
    // /
    // sub_elements: Arc<Mutex<Vec<Element>>>,
    ///
    ///
    reset_signal: Arc<Notify>,

    monitor_task_send: Sender<NamedTaskHandle>,
}

impl Class {
    ///
    ///
    pub fn new(builder: &ClassBuilder, notification_channel: Sender<Notification>) -> Self {
        Class {
            logger: builder.instance.logger.new_for_class(&builder.topic),
            instance: builder.instance.clone(),
            topic: builder.topic.clone(),
            enabled: Arc::new(AtomicBool::new(true)),
            notification_channel: notification_channel, // sub_elements: Arc::new(Mutex::new(Vec::new())),
            reset_signal: builder.instance.reset_signal(),
            monitor_task_send: builder.instance.task_monitor_sender().clone(),
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

#[async_trait]
impl Container for Class {
    /// Override
    ///
    fn logger(&self) -> &Logger {
        &self.logger
    }

    /// Override
    ///
    fn reset_signal(&self) -> Arc<Notify> {
        self.reset_signal.clone()
    }

    /// Override
    ///
    fn trigger_reset_signal(&self) {
        self.reset_signal.notify_waiters();
    }

    /// Override
    ///
    fn create_class<N: Into<String>>(&mut self, name: N) -> ClassBuilder {
        ClassBuilder::new(
            Some(self.clone()),
            self.instance.clone(),
            format!("{}/{}", self.topic, name.into()),
            self.notification_channel.clone(),
        )
    }

    /// Override
    ///
    fn create_attribute<N: Into<String>>(&mut self, name: N) -> AttributeServerBuilder {
        AttributeServerBuilder::new(
            self.instance.engine.clone(),
            None,
            self.notification_channel.clone(),
        )
        .with_topic(format!("{}/{}", self.topic, name.into()))
    }

    /// Override
    ///
    async fn monitor_task(&self, name: String, task_handle: TaskHandle) {
        self.monitor_task_send
            .send((name, task_handle))
            .await
            .unwrap();
    }
}
