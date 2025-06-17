use crate::instance::server::generic::Responder;
use crate::instance::server::GenericAttributeServer;
use crate::Error;
use crate::Logger;
use crate::Notification;
use panduza::attribute::CallbackId;
use panduza::fbs::BooleanBuffer;
use panduza::task_monitor::NamedTaskHandle;
use tokio::sync::mpsc::Sender;
use zenoh::Session;

#[derive(Clone)]
///
///
pub struct BooleanAttributeServer {
    pub inner: GenericAttributeServer<BooleanBuffer>,
}

impl BooleanAttributeServer {
    /// Logger getter
    ///
    pub fn logger(&self) -> &Logger {
        self.inner.logger()
    }

    ///
    ///
    pub fn r#type() -> String {
        "boolean".to_string()
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

        Self { inner }
    }

    /// Set the value of the attribute
    ///
    pub async fn set<T>(&self, value: T) -> Result<(), Error>
    where
        T: Into<BooleanBuffer>,
    {
        self.inner.set(value).await
    }

    /// Add a callback that will be triggered when receiving BooleanBuffer messages
    /// Optionally, a condition can be provided to filter when the callback is triggered
    #[inline]
    pub async fn add_callback<F, C>(&self, callback: F, condition: Option<C>) -> CallbackId
    where
        F: Fn(
                Responder,
                BooleanBuffer,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            + Send
            + Sync
            + 'static,
        C: Fn(&BooleanBuffer) -> bool + Send + Sync + 'static,
    {
        self.inner.add_callback(callback, condition).await
    }

    /// Remove a callback by its ID
    ///
    #[inline]
    pub async fn remove_callback(&self, callback_id: CallbackId) -> bool {
        self.inner.remove_callback(callback_id).await
    }

    ///
    ///
    pub async fn trigger_alert<T: Into<String>>(&self, message: T) {
        self.inner.trigger_alert(message).await;
    }
}
