use crate::log_trace;
use crate::runtime::notification::attribute::AttributeMode;
use crate::runtime::notification::EnablementNotification;
use crate::tracing::Logger;
use crate::AlertNotification;
use crate::AttributeBuilder;
use crate::Error;
use crate::MessageClient;
use crate::MessageCodec;
use crate::MessageDispatcher;
use crate::MessageHandler;
use crate::Notification;
use async_trait::async_trait;
use bytes::Bytes;
use rumqttc::QoS;
use std::sync::Arc;
use std::sync::Weak;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio::sync::Notify;

///
///
///
#[derive(Clone)]
pub struct AttServer<TYPE: MessageCodec> {
    /// Local logger
    ///
    pub logger: Logger,

    ///
    ///
    enabled: bool,

    /// Reactor message dispatcher
    /// (to attach this attribute to the incoming messages)
    message_dispatcher: Weak<Mutex<MessageDispatcher>>,

    ///
    /// The message client (MQTT)
    ///
    pub message_client: MessageClient,

    /// The topic of the attribute
    ///
    pub topic: String,

    ///
    /// New received messages are stored in this queue
    /// User can 'pop' them in its event callback to that every messages
    ///
    pub in_queue: Vec<TYPE>,

    ///
    /// Last popped value by the user
    ///
    pub last_popped_value: Option<TYPE>,

    ///
    /// Input notifier, alert when a new message has arrived in hte queue
    ///
    pub in_notifier: Arc<Notify>,

    ///
    /// The topic for 'att' topic to send data to user
    ///
    topic_att: String,

    ///
    /// Requested value of the attribute (set by the user)
    ///
    requested_value: Option<TYPE>,

    ///
    ///
    ///
    _mode: AttributeMode,

    r_notifier: Option<Sender<Notification>>,
}

impl<TYPE: MessageCodec> AttServer<TYPE> {
    ///
    /// Initialize the attribute
    /// Register the attribute on the message dispatcher then subscribe to att topic
    ///
    pub async fn init(&self, attribute: Arc<Mutex<dyn MessageHandler>>) -> Result<(), Error> {
        self.register(attribute).await?;
        self.subscribe().await
    }

    ///
    /// Send a notification to the underscore device to raise an alert
    ///
    pub fn send_alert(&self, message: String) {
        if let Some(r_notifier) = self.r_notifier.clone() {
            r_notifier
                .try_send(AlertNotification::new(self.topic.clone(), message).into())
                .unwrap();
        }
    }

    ///
    /// Subscribe to the topic
    ///
    pub async fn subscribe(&self) -> Result<(), Error> {
        // no need to store the att topic
        let topic_att = format!("{}/cmd", self.topic);
        self.message_client
            .subscribe(topic_att, QoS::AtMostOnce)
            .await
            .map_err(|e| Error::MessageAttributeSubscribeError(e.to_string()))
    }

    ///
    /// Register the attribute to the reactor
    ///
    pub async fn register(&self, attribute: Arc<Mutex<dyn MessageHandler>>) -> Result<(), Error> {
        // no need to store the att topic
        let topic_att = format!("{}/cmd", self.topic);
        self.message_dispatcher
            .upgrade()
            .ok_or(Error::InternalPointerUpgrade)?
            .lock()
            .await
            .register_message_attribute(topic_att, attribute);
        Ok(())
    }

    ///
    /// Get the value of the attribute
    /// If None, the first value is not yet received
    ///
    pub fn pop_cmd(&mut self) -> Option<TYPE> {
        if self.in_queue.is_empty() {
            None
        } else {
            let element = Some(self.in_queue.remove(0));
            self.last_popped_value = element.clone();
            element
        }
    }

    ///
    /// Get the value of the attribute
    /// If None, the first value is not yet received
    ///
    pub fn get_last_cmd(&self) -> Option<TYPE> {
        return self.last_popped_value.clone();
    }

    ///
    /// Clone the change notifier
    ///
    pub fn in_notifier(&self) -> Arc<Notify> {
        self.in_notifier.clone()
    }

    /// Set the value of the attribute
    ///
    pub async fn set(&mut self, new_value: TYPE) -> Result<(), Error> {
        // // Do not go further if the value is already set
        // if let Some(current_value) = self.value {
        //     if current_value == new_value {
        //         return Ok(());
        //     }
        // }

        // Set the requested value and publish the request
        self.requested_value = Some(new_value);
        match self.requested_value.clone() {
            Some(requested_value) => {
                self.publish(requested_value.into_message_payload()?)
                    .await?;
            }
            None => {
                return Err(Error::Wtf);
            }
        }

        Ok(())
    }

    /// Publish a command
    ///
    pub async fn publish<V>(&self, value: V) -> Result<(), Error>
    where
        V: Into<Vec<u8>>,
    {
        let value = value.into();
        let pyl_size = value.len();

        self.message_client
            .publish(&self.topic_att, QoS::AtMostOnce, true, value)
            .await
            .map_err(|e| Error::PublishError {
                topic: self.topic_att.clone(),
                pyl_size: pyl_size,
                cause: e.to_string(),
            })
    }

    /// Request attribute server disabling
    ///
    pub async fn change_enablement(&mut self, enabled: bool) -> Result<(), Error> {
        //
        // TRACE
        log_trace!(
            self.logger,
            "enablement change requested ! {:?} -> {:?}",
            self.enabled,
            enabled
        );

        //
        // Do action
        self.enabled = enabled;

        //
        // Send a notification if possible
        if let Some(notification_sender) = self.r_notifier.clone() {
            notification_sender
                .try_send(EnablementNotification::new(&self.topic, self.enabled).into())
                .map_err(|e| {
                    Error::InternalLogic(format!("fail to push platform notification ({:?})", e))
                })
        } else {
            Ok(())
        }
    }
}

#[async_trait]
impl<TYPE: MessageCodec> MessageHandler for AttServer<TYPE> {
    ///
    /// On message, just deserialize then push into the fifo
    ///
    async fn on_message(&mut self, data: &Bytes) -> Result<(), Error> {
        let in_value = TYPE::from_message_payload(data)?;
        self.in_queue.push(in_value);
        self.in_notifier.notify_waiters();
        Ok(())
    }
}

///
/// Allow creation from the builder
///
impl<TYPE: MessageCodec> From<AttributeBuilder> for AttServer<TYPE> {
    fn from(builder: AttributeBuilder) -> Self {
        let topic = builder.topic.as_ref().unwrap().clone();
        Self {
            logger: Logger::new_for_attribute_from_topic(topic.clone()),
            enabled: true, // enabled by default
            message_dispatcher: builder.message_dispatcher,
            message_client: builder.message_client,
            topic: topic.clone(),
            in_queue: Vec::new(),
            last_popped_value: None,
            in_notifier: Arc::new(Notify::new()),
            topic_att: format!("{}/att", topic.clone()),
            requested_value: None,
            _mode: builder.mode.unwrap(),
            r_notifier: builder.r_notifier,
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[macro_export]
// Macro that generate generic function for all att servers
//
macro_rules! generic_att_server_methods {
    () => {
        /// Logger getter
        ///
        pub fn logger(&self) -> &Logger {
            &self.logger
        }

        /// Bloc until at least a command is received
        ///
        pub async fn wait_commands(&self) {
            let in_notifier = self.inner.lock().await.in_notifier();
            in_notifier.notified().await
        }

        /// Bloc until at least a command is received then execute the 'function'
        ///
        pub async fn wait_commands_then<F>(&self, function: F) -> Result<(), Error>
        where
            F: Future<Output = Result<(), Error>> + Send + 'static,
        {
            let in_notifier = self.inner.lock().await.in_notifier();
            in_notifier.notified().await;
            function.await
        }

        ///
        ///
        pub async fn send_alert<T: Into<String>>(&self, message: T) {
            self.inner.lock().await.send_alert(message.into());
        }

        /// Request attribute server enablement
        ///
        pub async fn change_enablement(&mut self, enabled: bool) -> Result<(), Error> {
            self.inner.lock().await.change_enablement(enabled).await
        }

        /// Request attribute server enablement
        ///
        pub async fn enable(&mut self) -> Result<(), Error> {
            self.inner.lock().await.change_enablement(true).await
        }

        /// Request attribute server disablement
        ///
        pub async fn disable(&mut self) -> Result<(), Error> {
            self.inner.lock().await.change_enablement(false).await
        }
    };
}
