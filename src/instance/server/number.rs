use crate::instance::server::StdObjAttributeServer;
use crate::Error;
use crate::Logger;
use crate::Notification;
use panduza::attribute::CallbackId;
use panduza::fbs::NumberBuffer;
use panduza::task_monitor::NamedTaskHandle;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use zenoh::Session;

#[derive(Clone)]
///
/// NumberAttributeServer provides a server for numeric attributes
///
pub struct NumberAttributeServer {
    pub inner: Arc<StdObjAttributeServer<NumberBuffer>>,
}

impl NumberAttributeServer {
    /// Logger getter
    ///
    pub fn logger(&self) -> &Logger {
        self.inner.logger()
    }

    ///
    /// Create a new NumberAttributeServer
    ///
    pub async fn new(
        session: Session,
        topic: String,
        task_monitor_sender: Sender<NamedTaskHandle>,
        notification_channel: Sender<Notification>,
    ) -> Self {
        let inner = StdObjAttributeServer::<NumberBuffer>::new(
            session,
            topic,
            task_monitor_sender,
            notification_channel,
        )
        .await;

        Self {
            inner: Arc::new(inner),
        }
    }

    /// Set the value of the attribute
    ///
    pub async fn set<V>(&self, value: V) -> Result<(), Error>
    where
        V: Into<f64>,
    {
        let buffer = NumberBuffer::builder()
            .with_value(value.into())
            .with_source(0)
            .with_random_sequence()
            .build()
            .unwrap();
        self.inner.set(buffer).await
    }

    /// Set the value with specific unit
    ///
    pub async fn with_unit<V>(&self, value: V) -> Result<(), Error>
    where
        V: Into<f64>,
    {
        let buffer = NumberBuffer::builder()
            .with_value(value.into())
            .with_source(0)
            .with_random_sequence()
            .build()
            .unwrap();
        self.inner.set(buffer).await
    }

    /// Set the value with decimals
    ///
    pub async fn with_decimals<V>(&self, value: V, _decimals: u8) -> Result<(), Error>
    where
        V: Into<f64>,
    {
        let buffer = NumberBuffer::builder()
            .with_value(value.into())
            .with_source(0)
            .with_random_sequence()
            .build()
            .unwrap();
        self.inner.set(buffer).await
    }

    /// Set the value with range constraints
    ///
    pub async fn with_range<V>(&self, value: V, _min: f64, _max: f64) -> Result<(), Error>
    where
        V: Into<f64>,
    {
        let buffer = NumberBuffer::builder()
            .with_value(value.into())
            .with_source(0)
            .with_random_sequence()
            .build()
            .unwrap();
        self.inner.set(buffer).await
    }

    /// Set the value with whitelist constraints
    ///
    pub async fn with_whitelist<V>(&self, value: V, _whitelist: Vec<f64>) -> Result<(), Error>
    where
        V: Into<f64>,
    {
        let buffer = NumberBuffer::builder()
            .with_value(value.into())
            .with_source(0)
            .with_random_sequence()
            .build()
            .unwrap();
        self.inner.set(buffer).await
    }

    /// Ajoute un callback sans condition (toujours déclenché)
    ///
    #[inline]
    pub fn add_callback<F>(&self, callback: F) -> impl std::future::Future<Output = CallbackId> + '_
    where
        F: Fn(NumberBuffer) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            + Send
            + Sync
            + 'static,
    {
        self.inner
            .add_callback(callback, Option::<fn(&NumberBuffer) -> bool>::None)
    }

    /// Ajoute un callback avec une condition personnalisée
    ///
    #[inline]
    pub fn add_callback_with_condition<F, C>(
        &self,
        callback: F,
        condition: C,
    ) -> impl std::future::Future<Output = CallbackId> + '_
    where
        F: Fn(NumberBuffer) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            + Send
            + Sync
            + 'static,
        C: Fn(&NumberBuffer) -> bool + Send + Sync + 'static,
    {
        self.inner.add_callback(callback, Some(condition))
    }

    /// Remove a callback by its ID
    ///
    #[inline]
    pub fn remove_callback(
        &self,
        callback_id: CallbackId,
    ) -> impl std::future::Future<Output = bool> + '_ {
        self.inner.remove_callback(callback_id)
    }

    ///
    /// Trigger an alert
    ///
    #[inline]
    pub fn trigger_alert<T: Into<String> + 'static>(
        &self,
        message: T,
    ) -> impl std::future::Future<Output = ()> + '_ {
        self.inner.trigger_alert(message)
    }
}
