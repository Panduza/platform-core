use super::server::boolean::BooleanAttributeServer;
use super::server::bytes::BytesAttributeServer;
use super::server::json::JsonAttributeServer;
use super::server::notification_v0::NotificationAttributeServer;
use super::server::number::NumberAttributeServer;
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
use panduza::task_monitor::NamedTaskHandle;
use tokio::sync::mpsc::Sender;
use zenoh::handlers::FifoChannelHandler;
use zenoh::pubsub::Publisher;
use zenoh::pubsub::Subscriber;
use zenoh::sample::Sample;

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

    // ------------------------------------------------------------------------

    /// Send a notification to the platform
    ///
    async fn send_creation_notification(&self) {
        //
        // Debug
        // println!(
        //     "send_creation_notification '{}' !",
        //     self.topic.as_ref().unwrap()
        // );

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

    // ------------------------------------------------------------------------

    ///
    ///
    async fn common_ops(
        &self,
        cmd_channel_size: usize,
    ) -> (Subscriber<FifoChannelHandler<Sample>>, Publisher) {
        let topic = self.topic.as_ref().unwrap();

        let topic_prefixless = if let Some(namespace) = self.engine.namespace.as_ref() {
            // topic.strip_prefix(namespace).unwrap_or(topic)
            // format!(
            //     "*{}",
            //     topic.strip_prefix(namespace).unwrap_or(topic.as_str())
            // )

            if namespace.is_empty() {
                topic.to_string()
            } else {
                format!(
                    "*{}",
                    topic.strip_prefix(namespace).unwrap_or(topic.as_str())
                )
            }
        } else {
            topic.to_string()
        };

        // let topic = if let Some(namespace) = self.engine.namespace.as_ref() {
        //     if namespace.is_empty() {
        //         topic.strip_prefix("/").unwrap_or(topic).to_string()
        //     } else {
        //         topic.to_string()
        //     }
        // } else {
        //     topic.to_string()
        // };

        // if let Some(namespace) = self.engine.namespace.as_ref() {
        //     topic_prefixless = topic_prefixless.strip_prefix("*");
        // }

        let cmd_receiver = self
            .engine
            .register_listener(format!("{}/cmd", topic_prefixless), 50)
            .await;

        let att_publisher = self
            .engine
            .register_publisher(format!("{}/att", topic))
            .await
            .unwrap();

        (cmd_receiver, att_publisher)
    }

    // ------------------------------------------------------------------------

    /// BOOLEAN
    ///
    pub async fn start_as_boolean(mut self) -> Result<BooleanAttributeServer, Error> {
        self.r#type = Some("boolean".to_string());
        self.send_creation_notification().await;
        let att = BooleanAttributeServer::new(
            self.engine.session,
            self.topic.unwrap(),
            self.task_monitor_sender,
            self.notification_channel,
        )
        .await;
        Ok(att)
    }

    // ------------------------------------------------------------------------

    /// NUMBER
    ///
    pub async fn start_as_number(mut self) -> Result<NumberAttributeServer, Error> {
        self.r#type = Some("number".to_string());
        self.send_creation_notification().await;
        let att = NumberAttributeServer::new(
            self.engine.session,
            self.topic.unwrap(),
            self.task_monitor_sender,
            self.notification_channel,
        )
        .await;
        Ok(att)
    }

    // ------------------------------------------------------------------------

    /// STRING
    ///
    pub async fn start_as_string(mut self) -> Result<StringAttributeServer, Error> {
        self.r#type = Some("string".to_string());
        self.send_creation_notification().await;
        let att = StringAttributeServer::new(
            self.engine.session,
            self.topic.unwrap(),
            self.task_monitor_sender,
            self.notification_channel,
        )
        .await;
        Ok(att)
    }

    // ------------------------------------------------------------------------

    /// BYTES
    ///
    pub async fn start_as_bytes(mut self) -> Result<BytesAttributeServer, Error> {
        self.r#type = Some("bytes".to_string());
        self.send_creation_notification().await;
        let att = BytesAttributeServer::new(
            self.engine.session,
            self.topic.unwrap(),
            self.task_monitor_sender,
            self.notification_channel,
        )
        .await;
        Ok(att)
    }

    // ------------------------------------------------------------------------

    /// NOTIFICATION
    ///
    pub async fn __start_as_notification(mut self) -> Result<NotificationAttributeServer, Error> {
        let topic = self.topic.as_ref().unwrap();
        self.r#type = Some(NotificationAttributeServer::r#type());
        let (cmd_receiver, att_publisher) = self.common_ops(50).await;
        let att = NotificationAttributeServer::new(
            self.engine.session.clone(),
            topic.clone(),
            cmd_receiver,
            self.task_monitor_sender.clone(),
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
            self.engine.session.clone(),
            topic.clone(),
            cmd_receiver,
            self.task_monitor_sender.clone(),
        )
        .await;
        Ok(att)
    }

    /// TRIGGER
    ///
    pub async fn start_as_trigger(mut self) -> Result<TriggerAttributeServer, Error> {
        let topic = self.topic.as_ref().unwrap();
        self.r#type = Some(TriggerAttributeServer::r#type());
        let (cmd_receiver, att_publisher) = self.common_ops(50).await;
        let att = TriggerAttributeServer::new(
            self.engine.session.clone(),
            topic.clone(),
            cmd_receiver,
            self.task_monitor_sender.clone(),
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
            self.engine.session.clone(),
            topic.clone(),
            cmd_receiver,
            self.task_monitor_sender.clone(),
        );
        Ok(att)
    }

    ///
    ///
    pub async fn start_as_json(mut self) -> Result<JsonAttributeServer, Error> {
        // //
        // //
        // self.r#type = Some(JsonAttributeServer::r#type());

        // //
        // //
        // self.send_creation_notification().await;

        // let topic = self.topic.unwrap();

        // let cmd_receiver = self
        //     .engine
        //     .register_listener(format!("{}/cmd", topic), 50)
        //     .await;

        // let att_publisher = self
        //     .engine
        //     .register_publisher(format!("{}/att", topic))
        //     .await
        //     .unwrap();

        // //
        // //
        // let att = JsonAttributeServer::new(
        //     self.engine.session.clone(),
        //     topic,
        //     cmd_receiver,
        //     self.task_monitor_sender,
        // );

        // Ok(att)

        let topic: &String = self.topic.as_ref().unwrap();
        self.r#type = Some(JsonAttributeServer::r#type());
        let (cmd_receiver, att_publisher) = self.common_ops(50).await;
        let att = JsonAttributeServer::new(
            self.engine.session.clone(),
            topic.clone(),
            cmd_receiver,
            self.task_monitor_sender.clone(),
            self.notification_channel.clone(),
        )
        .await;
        Ok(att)
    }
}
