use crate::instance::server::GenericAttributeServer;
use crate::Error;
use crate::Logger;
use crate::Notification;
use panduza::attribute::CallbackId;
use panduza::fbs::BooleanBuffer;
use panduza::task_monitor::NamedTaskHandle;
use panduza::PanduzaBuffer;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use zenoh::Session;

#[derive(Clone)]
///
///
pub struct BooleanAttributeServer {
    pub inner: Arc<GenericAttributeServer<BooleanBuffer>>,
}

impl BooleanAttributeServer {
    /// Logger getter
    ///
    pub fn logger(&self) -> &Logger {
        self.inner.logger()
    }

    ///
    ///
    pub async fn new(
        session: Session,
        topic: String,
        task_monitor_sender: Sender<NamedTaskHandle>,
        notification_channel: Sender<Notification>,
    ) -> Self {
        let inner = GenericAttributeServer::<BooleanBuffer>::new(
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
        V: Into<bool>,
    {
        let buffer = BooleanBuffer::from(value.into())
            .with_source(0)
            .with_random_sequence()
            .build()
            .unwrap();
        self.inner.set(buffer).await
    }

    ///
    ///
    pub async fn reply_to<T, V>(&self, command: &T, value: V)
    where
        T: PanduzaBuffer,
        V: Into<bool>,
    {
        self.inner.reply_to(command, value.into()).await;
    }

    /// Ajoute un callback sans condition (toujours déclenché)
    ///
    #[inline]
    pub fn add_callback<F>(&self, callback: F) -> impl std::future::Future<Output = CallbackId> + '_
    where
        F: Fn(BooleanBuffer) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            + Send
            + Sync
            + 'static,
    {
        self.inner
            .add_callback(callback, Option::<fn(&BooleanBuffer) -> bool>::None)
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
        F: Fn(BooleanBuffer) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            + Send
            + Sync
            + 'static,
        C: Fn(&BooleanBuffer) -> bool + Send + Sync + 'static,
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
    ///
    #[inline]
    pub fn trigger_alert<T: Into<String> + 'static>(
        &self,
        message: T,
    ) -> impl std::future::Future<Output = ()> + '_ {
        self.inner.trigger_alert(message)
    }
}
