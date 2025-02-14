use crate::{Class, ClassNotification, Engine};

use super::Instance;

pub struct ClassBuilder {
    /// Parent class if any
    ///
    parent_class: Option<Class>,

    //
    pub reactor: Engine,
    ///
    pub device: Instance,
    ///
    /// Option because '_' device will not provide one
    ///
    // pub device_dyn_info: Option<ThreadSafeInfoDynamicDeviceStatus>,
    // pub r_notifier: Option<Sender<Notification>>,
    ///
    pub topic: String,

    pub tags: Vec<String>,
}

impl ClassBuilder {
    pub fn new<N: Into<String>>(
        parent_class: Option<Class>,
        reactor: Engine, // deprecated because acces through device
        device: Instance,
        // device_dyn_info: Option<ThreadSafeInfoDynamicDeviceStatus>,
        topic: N,
    ) -> Self {
        Self {
            parent_class: parent_class,
            reactor: reactor,
            device: device,
            // device_dyn_info: device_dyn_info,
            topic: topic.into(),
            tags: Vec::new(),
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
        if let Some(r_notifier) = self.device.r_notifier.clone() {
            r_notifier
                .try_send(ClassNotification::new(bis, self.tags.clone()).into())
                .unwrap();
        }
        // insert in status
        let class = Class::new(&self);

        //
        // Attach the attribute to its parent class if exist
        if let Some(mut parent_class) = self.parent_class {
            parent_class
                .push_sub_element(class.clone_as_element())
                .await;
        }

        class
    }
}
