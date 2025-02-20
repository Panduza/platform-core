use super::server::boolean::BooleanAttributeServer;
use super::server::json::JsonAttributeServer;
use super::server::si::SiAttributeServer;
use crate::instance::class::Class;
use crate::runtime::notification::attribute::AttributeMode;
use crate::AttributeNotification;
use crate::Engine;
use crate::Error;
use crate::Notification;
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
///
/// Object that allow to build an generic attribute
///
pub struct AttributeServerBuilder {
    /// Platform connection engine
    ///
    engine: Engine,

    /// Parent class if any
    ///
    parent_class: Option<Class>,

    /// Topic of the attribute
    pub topic: Option<String>,

    ///
    /// Attribute Settings
    ///
    pub settings: Option<serde_json::Value>,

    pub mode: Option<AttributeMode>,
    pub r#type: Option<String>,

    pub info: Option<String>,

    ///
    ///
    notification_channel: Sender<Notification>,
}

impl AttributeServerBuilder {
    /// Create a new builder
    ///
    pub fn new(
        engine: Engine,
        parent_class: Option<Class>,
        notification_channel: Sender<Notification>,
    ) -> Self {
        Self {
            engine,
            parent_class,
            topic: None,
            settings: None,
            mode: None,
            r#type: None,
            info: None,
            notification_channel: notification_channel,
        }
    }

    /// Attach a topic
    ///
    pub fn with_topic<T: Into<String>>(mut self, topic: T) -> Self {
        self.topic = Some(topic.into());
        self
    }

    /// Attach settings to the attribute
    ///
    pub fn with_settings(mut self, settings: serde_json::Value) -> Self {
        self.settings = Some(settings);
        self
    }

    /// Set the Read Only mode to this attribute
    ///
    pub fn with_ro(mut self) -> Self {
        self.mode = Some(AttributeMode::ReadOnly);
        self
    }

    /// Set the Write Only mode to this attribute
    ///
    pub fn with_wo(mut self) -> Self {
        self.mode = Some(AttributeMode::WriteOnly);
        self
    }

    /// Set the Write Read mode to this attribute
    ///
    pub fn with_rw(mut self) -> Self {
        self.mode = Some(AttributeMode::ReadWrite);
        self
    }

    ///
    ///
    pub fn with_info<T: Into<String>>(mut self, info: T) -> Self {
        self.info = Some(info.into());
        self
    }

    ///
    ///
    pub async fn start_as_boolean(mut self) -> Result<BooleanAttributeServer, Error> {
        //
        //
        self.r#type = Some(BooleanAttributeServer::r#type());

        //
        //
        self.send_creation_notification().await;

        let topic = self.topic.unwrap();

        let cmd_receiver: tokio::sync::mpsc::Receiver<bytes::Bytes> = self
            .engine
            .register_listener(format!("{}/cmd", topic), 50)
            .await
            .unwrap();

        let att_publisher = self
            .engine
            .register_publisher(format!("{}/att", topic), true)
            .unwrap();

        //
        //
        let att = BooleanAttributeServer::new(topic, cmd_receiver, att_publisher);

        // //
        // // Attach the attribute to its parent class if exist
        // if let Some(mut parent_class) = self.parent_class {
        //     parent_class.push_sub_element(att.clone_as_element()).await;
        // }

        Ok(att)
    }

    ///
    ///
    pub async fn start_as_json(mut self) -> Result<JsonAttributeServer, Error> {
        //
        //
        self.r#type = Some(JsonAttributeServer::r#type());

        //
        //
        self.send_creation_notification().await;

        let topic = self.topic.unwrap();

        let cmd_receiver: tokio::sync::mpsc::Receiver<bytes::Bytes> = self
            .engine
            .register_listener(format!("{}/cmd", topic), 50)
            .await
            .unwrap();

        let att_publisher = self
            .engine
            .register_publisher(format!("{}/att", topic), true)
            .unwrap();

        //
        //
        let att = JsonAttributeServer::new(topic, cmd_receiver, att_publisher);

        // //
        // // Attach the attribute to its parent class if exist
        // if let Some(mut parent_class) = self.parent_class {
        //     parent_class.push_sub_element(att.clone_as_element()).await;
        // }

        Ok(att)
    }

    ///
    ///
    pub async fn start_as_si<N: Into<String>>(
        mut self,
        unit: N,
        min: f64,
        max: f64,
        decimals: usize,
    ) -> Result<SiAttributeServer, Error> {
        //
        //
        self.r#type = Some(SiAttributeServer::r#type());

        //
        //
        self.send_creation_notification().await;

        let topic = self.topic.unwrap();

        let cmd_receiver: tokio::sync::mpsc::Receiver<bytes::Bytes> = self
            .engine
            .register_listener(format!("{}/cmd", topic), 50)
            .await
            .unwrap();

        let att_publisher = self
            .engine
            .register_publisher(format!("{}/att", topic), true)
            .unwrap();

        //
        //
        let att = SiAttributeServer::new(
            topic,
            cmd_receiver,
            att_publisher,
            unit.into(),
            min,
            max,
            decimals,
        );

        // //
        // // Attach the attribute to its parent class if exist
        // if let Some(mut parent_class) = self.parent_class {
        //     parent_class.push_sub_element(att.clone_as_element()).await;
        // }

        Ok(att)
    }

    ///
    ///
    async fn send_creation_notification(&self) {
        //
        // Debug
        // println!("channel send_creation_notification !!");

        //
        //
        let bis = self.topic.clone().unwrap();
        self.notification_channel
            .send(
                AttributeNotification::new(
                    bis,
                    self.r#type.clone().unwrap(),
                    self.mode.clone().unwrap(),
                    self.info.clone(),
                    self.settings.clone(),
                )
                .into(),
            )
            .await
            .unwrap();
    }
}
