use crate::instance::class::Class;
use crate::runtime::notification::attribute::AttributeMode;
use crate::Engine;
use crate::Error;
use serde_json::json;
use std::sync::Weak;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use super::attribute::server_boolean::BooleanAttributeServer;
use super::attribute::server_json::JsonAttributeServer;

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
    // ///
    // pub r_notifier: Option<Sender<Notification>>,
}

impl AttributeServerBuilder {
    /// Create a new builder
    pub fn new(
        engine: Engine,
        parent_class: Option<Class>,
        // message_client: MessageClient,
        // message_dispatcher: Weak<Mutex<MessageDispatcher>>,
        // r_notifier: Option<Sender<Notification>>,
    ) -> Self {
        Self {
            engine,
            parent_class,
            // message_client,
            // message_dispatcher,
            // r_notifier,
            topic: None,
            settings: None,
            mode: None,
            r#type: None,
            info: None,
        }
    }

    /// Attach a topic
    ///
    pub fn with_topic<T: Into<String>>(mut self, topic: T) -> Self {
        self.topic = Some(topic.into());
        self
    }

    ///
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

        // self.send_creation_notification();

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

        // self.send_creation_notification();

        // //
        // // Attach the attribute to its parent class if exist
        // if let Some(mut parent_class) = self.parent_class {
        //     parent_class.push_sub_element(att.clone_as_element()).await;
        // }

        Ok(att)
    }

    // ///
    // ///
    // ///
    // pub async fn finish_as_si<N: Into<String>>(
    //     mut self,
    //     unit: N,
    //     min: f64,
    //     max: f64,
    //     decimals: usize,
    // ) -> Result<SiAttServer, Error> {
    //     self.r#type = Some(SiAttServer::r#type());
    //     let unit_string = unit.into();
    //     self.settings = Some(json!(
    //         {
    //             "unit": unit_string.clone(),
    //             "min": min,
    //             "max": max,
    //             "decimals": decimals,
    //         }
    //     ));
    //     let att = SiAttServer::new(self.clone(), unit_string, min, max, decimals);
    //     att.inner.lock().await.init(att.inner.clone()).await?;
    //     self.send_creation_notification();
    //     Ok(att)
    // }

    // ///
    // /// Finish attribute building and configure it with 'string' type.
    // ///
    // pub async fn finish_as_string(mut self) -> Result<StringAttServer, Error> {
    //     self.r#type = Some(StringAttServer::r#type());
    //     let att = StringAttServer::new(self.clone());
    //     att.inner.lock().await.init(att.inner.clone()).await?;
    //     self.send_creation_notification();
    //     Ok(att)
    // }

    // /// Finish attribute building and configure it with 'enum' type.
    // ///
    // pub async fn finish_as_enum<S: Into<String>>(
    //     mut self,
    //     choices: Vec<S>,
    // ) -> Result<EnumAttServer, Error> {
    //     self.r#type = Some(EnumAttServer::r#type());

    //     //
    //     // Convert choices to Vec<String>
    //     let choices: Vec<String> = choices.into_iter().map(Into::into).collect();

    //     //
    //     // Provide enum settings
    //     self.settings = Some(json!(
    //         {
    //             "choices": choices.clone(),
    //         }
    //     ));

    //     //
    //     // Create server object
    //     let att = EnumAttServer::new(self.clone(), choices);
    //     att.inner.lock().await.init(att.inner.clone()).await?;
    //     self.send_creation_notification();
    //     Ok(att)
    // }

    // ///
    // /// Finish attribute building and configure it with 'json' type.
    // ///
    // pub async fn finish_as_json(mut self) -> Result<JsonAttServer, Error> {
    //     self.r#type = Some(JsonAttServer::r#type());
    //     let att = JsonAttServer::new(self.clone());

    //     att.inner.lock().await.init(att.inner.clone()).await?;
    //     self.send_creation_notification();

    //     //
    //     // Attach the attribute to its parent class if exist
    //     if let Some(mut parent_class) = self.parent_class {
    //         parent_class.push_sub_element(att.clone_as_element()).await;
    //     }

    //     Ok(att)
    // }

    // ///
    // ///
    // pub async fn finish_as_number(mut self) -> Result<NumberAttServer, Error> {
    //     self.r#type = Some(NumberAttServer::r#type());
    //     let att = NumberAttServer::new(self.clone());
    //     att.inner.lock().await.init(att.inner.clone()).await?;
    //     self.send_creation_notification();
    //     Ok(att)
    // }

    // ///
    // ///
    // pub async fn finish_as_memory_command(mut self) -> Result<MemoryCommandAttServer, Error> {
    //     self.r#type = Some(MemoryCommandAttServer::r#type());
    //     let att = MemoryCommandAttServer::new(self.clone());
    //     att.inner.lock().await.init(att.inner.clone()).await?;
    //     self.send_creation_notification();
    //     Ok(att)
    // }

    // ///
    // ///
    // ///
    // fn send_creation_notification(&self) {
    //     //
    //     //
    //     let bis = self.topic.clone().unwrap();
    //     if let Some(r_notifier) = self.r_notifier.clone() {
    //         r_notifier
    //             .try_send(
    //                 AttributeNotification::new(
    //                     bis,
    //                     self.r#type.clone().unwrap(),
    //                     self.mode.clone().unwrap(),
    //                     self.info.clone(),
    //                     self.settings.clone(),
    //                 )
    //                 .into(),
    //             )
    //             .unwrap();
    //     }
    // }
}
