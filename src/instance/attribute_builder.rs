use super::server::boolean::BooleanAttributeServer;
use super::server::bytes::BytesAttributeServer;
use super::server::json::JsonAttributeServer;
use super::server::notification_v0::NotificationAttributeServer;
use super::server::r#enum::EnumAttributeServer;
use super::server::sample::SampleAttributeServer;
use super::server::si::SiAttributeServer;
use super::server::status_v0::StatusAttributeServer;
use super::server::string::StringAttributeServer;
use super::server::trigger_v0::TriggerAttributeServer;
use super::server::vector_f32_v0::VectorF32AttributeServer;
use crate::instance::class::Class;
use crate::runtime::notification::attribute::AttributeMode;
use crate::AttributeNotification;
use crate::Engine;
use crate::Error;
use crate::Notification;
use panduza::pubsub::Publisher;
use panduza::task_monitor::NamedTaskHandle;
use serde_json::json;
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

    /// Channel to send notifications
    ///
    notification_channel: Sender<Notification>,

    ///
    ///
    task_monitor_sender: Sender<NamedTaskHandle>,
}

impl AttributeServerBuilder {
    /// Create a new builder
    ///
    pub fn new(
        engine: Engine,
        parent_class: Option<Class>,
        notification_channel: Sender<Notification>,
        task_monitor_sender: Sender<NamedTaskHandle>,
    ) -> Self {
        Self {
            engine,
            parent_class,
            topic: None,
            settings: None,
            mode: Some(AttributeMode::ReadOnly),
            r#type: None,
            info: None,
            notification_channel: notification_channel,
            task_monitor_sender: task_monitor_sender,
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

    ///
    ///
    async fn common_ops(
        &self,
        cmd_channel_size: usize,
    ) -> (tokio::sync::mpsc::Receiver<bytes::Bytes>, Publisher) {
        //
        //
        self.send_creation_notification().await;

        let topic = self.topic.as_ref().unwrap();

        let cmd_receiver: tokio::sync::mpsc::Receiver<bytes::Bytes> = self
            .engine
            .register_listener(format!("{}/cmd", topic), 50)
            .await
            .unwrap();

        let att_publisher = self
            .engine
            .register_publisher(format!("{}/att", topic), true)
            .unwrap();

        (cmd_receiver, att_publisher)
    }

    /// BOOLEAN
    ///
    pub async fn start_as_boolean(mut self) -> Result<BooleanAttributeServer, Error> {
        let topic = self.topic.as_ref().unwrap();
        self.r#type = Some(BooleanAttributeServer::r#type());
        let (cmd_receiver, att_publisher) = self.common_ops(50).await;
        let att = BooleanAttributeServer::new(
            topic.clone(),
            cmd_receiver,
            att_publisher,
            self.task_monitor_sender,
            self.notification_channel.clone(),
        )
        .await;
        Ok(att)
    }

    /// ENUM
    ///
    pub async fn start_as_enum<S: Into<String>>(
        mut self,
        choices: Vec<S>,
    ) -> Result<EnumAttributeServer, Error> {
        let topic = self.topic.as_ref().unwrap();
        self.r#type = Some(EnumAttributeServer::r#type());

        let choices: Vec<String> = choices.into_iter().map(Into::into).collect();
        self.settings = Some(json!({
            "choices": choices.clone(),
        }));

        let (cmd_receiver, att_publisher) = self.common_ops(50).await;
        let att = EnumAttributeServer::new(
            topic.clone(),
            cmd_receiver,
            att_publisher,
            self.task_monitor_sender,
            choices.clone(),
        );
        Ok(att)
    }

    /// NOTIFICATION
    ///
    pub async fn __start_as_notification(mut self) -> Result<NotificationAttributeServer, Error> {
        let topic = self.topic.as_ref().unwrap();
        self.r#type = Some(NotificationAttributeServer::r#type());
        let (cmd_receiver, att_publisher) = self.common_ops(50).await;
        let att = NotificationAttributeServer::new(
            topic.clone(),
            cmd_receiver,
            att_publisher,
            self.task_monitor_sender,
        )
        .await;
        Ok(att)
    }

    /// STATUS
    ///
    pub async fn __start_as_status(mut self) -> Result<StatusAttributeServer, Error> {
        let topic = self.topic.as_ref().unwrap();
        self.r#type = Some(StatusAttributeServer::r#type());
        let (cmd_receiver, att_publisher) = self.common_ops(50).await;
        let att = StatusAttributeServer::new(
            topic.clone(),
            cmd_receiver,
            att_publisher,
            self.task_monitor_sender,
        )
        .await;
        Ok(att)
    }

    /// SAMPLE
    ///
    pub async fn start_as_sample(mut self) -> Result<SampleAttributeServer, Error> {
        let topic = self.topic.as_ref().unwrap();
        self.r#type = Some(SampleAttributeServer::r#type());
        let (cmd_receiver, att_publisher) = self.common_ops(50).await;
        let att = SampleAttributeServer::new(topic.clone(), cmd_receiver, att_publisher);
        Ok(att)
    }

    /// TRIGGER
    ///
    pub async fn start_as_trigger(mut self) -> Result<TriggerAttributeServer, Error> {
        let topic = self.topic.as_ref().unwrap();
        self.r#type = Some(TriggerAttributeServer::r#type());
        let (cmd_receiver, att_publisher) = self.common_ops(50).await;
        let att = TriggerAttributeServer::new(
            topic.clone(),
            cmd_receiver,
            att_publisher,
            self.task_monitor_sender,
        );
        Ok(att)
    }

    /// VECTOR_F32
    ///
    pub async fn start_as_vector_f32(mut self) -> Result<VectorF32AttributeServer, Error> {
        let topic = self.topic.as_ref().unwrap();
        self.r#type = Some(VectorF32AttributeServer::r#type());
        let (cmd_receiver, att_publisher) = self.common_ops(50).await;
        let att = VectorF32AttributeServer::new(
            topic.clone(),
            cmd_receiver,
            att_publisher,
            self.task_monitor_sender,
        );
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
        let att =
            JsonAttributeServer::new(topic, cmd_receiver, att_publisher, self.task_monitor_sender);

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
        let topic = self.topic.as_ref().unwrap();
        self.r#type = Some(SiAttributeServer::r#type());

        let unit = unit.into();
        self.settings = Some(json!({
            "unit": unit.clone(),
            "min": min,
            "max": max,
            "decimals": decimals,
        }));

        let (cmd_receiver, att_publisher) = self.common_ops(50).await;
        let att = SiAttributeServer::new(
            topic.clone(),
            cmd_receiver,
            att_publisher,
            unit.clone(),
            min,
            max,
            decimals,
            self.task_monitor_sender,
        )
        .await;
        Ok(att)
    }

    ///
    ///
    pub async fn start_as_string(mut self) -> Result<StringAttributeServer, Error> {
        //
        //
        self.r#type = Some(StringAttributeServer::r#type());

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
        let att = StringAttributeServer::new(
            topic,
            cmd_receiver,
            att_publisher,
            self.task_monitor_sender,
        )
        .await;

        // //
        // // Attach the attribute to its parent class if exist
        // if let Some(mut parent_class) = self.parent_class {
        //     parent_class.push_sub_element(att.clone_as_element()).await;
        // }

        Ok(att)
    }

    /// BYTES
    ///
    pub async fn start_as_bytes(mut self) -> Result<BytesAttributeServer, Error> {
        //
        //
        self.r#type = Some(BytesAttributeServer::r#type());

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
        let att =
            BytesAttributeServer::new(topic, cmd_receiver, att_publisher, self.task_monitor_sender)
                .await;

        // //
        // // Attach the attribute to its parent class if exist
        // if let Some(mut parent_class) = self.parent_class {
        //     parent_class.push_sub_element(att.clone_as_element()).await;
        // }

        Ok(att)
    }
}
