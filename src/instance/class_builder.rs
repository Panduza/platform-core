use tokio::sync::mpsc::Sender;

use crate::{ClassNotification, Notification};

use super::{class::Class, Instance};

pub struct ClassBuilder {
    /// Parent class if any
    ///
    pub parent_class: Option<Class>,

    ///
    ///
    pub instance: Instance,

    ///
    /// Option because '_' instance will not provide one
    ///
    // pub device_dyn_info: Option<ThreadSafeInfoDynamicDeviceStatus>,
    // pub r_notifier: Option<Sender<Notification>>,
    ///
    pub topic: String,

    pub tags: Vec<String>,

    ///
    ///
    notification_channel: Sender<Notification>,
}

impl ClassBuilder {
    pub fn new<N: Into<String>>(
        parent_class: Option<Class>,
        instance: Instance,
        // device_dyn_info: Option<ThreadSafeInfoDynamicDeviceStatus>,
        topic: N,
        notification_channel: Sender<Notification>,
    ) -> Self {
        Self {
            parent_class: parent_class,

            instance: instance,
            // device_dyn_info: device_dyn_info,
            topic: topic.into(),
            tags: Vec::new(),
            notification_channel: notification_channel,
        }
    }

    pub fn with_tag<T: Into<String>>(mut self, tag: T) -> Self {
        self.tags.push(tag.into());
        self
    }

    ///
    ///
    ///
    pub async fn finish(self) -> Class {
        let bis = self.topic.clone();

        //
        //
        self.notification_channel
            .send(ClassNotification::new(bis, self.tags.clone()).into())
            .await
            .unwrap();

        //
        let class = Class::new(&self, self.notification_channel.clone());

        //
        // Attach the attribute to its parent class if exist
        // if let Some(mut parent_class) = self.parent_class {
        //     parent_class
        //         .push_sub_element(class.clone_as_element())
        //         .await;
        // }

        class
    }
}
