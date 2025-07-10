use crate::instance::server::ro_stream::RoStreamAttributeServer;
// use crate::Error;
use crate::Logger;
use crate::Notification;
// use panduza::fbs::NotificationBuffer;
// use panduza::fbs::NotificationType;
// use panduza::task_monitor::NamedTaskHandle;
// use panduza::PanduzaBuffer;
use std::sync::Arc;
// use std::sync::Mutex;
use tokio::sync::mpsc::Sender;
// use zenoh::handlers::FifoChannelHandler;
// use zenoh::pubsub::Subscriber;
// use zenoh::sample::Sample;
use zenoh::Session;

///
///
pub struct NotificationAttributeServer {
    ///
    ///
    pub inner: Arc<RoStreamAttributeServer>,
}

impl NotificationAttributeServer {
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
        notification_channel: Sender<Notification>,
    ) -> Self {
        // Initialize the inner implementation
        let inner = RoStreamAttributeServer::new(session, topic, notification_channel).await;
        Self {
            inner: Arc::new(inner),
        }
    }
}
