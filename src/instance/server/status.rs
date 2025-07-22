use crate::instance::server::std_obj::StdObjAttributeServer;
use crate::Error;
use crate::Logger;
use crate::Notification;
use panduza::fbs::StatusBuffer;
use panduza::task_monitor::NamedTaskHandle;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use zenoh::Session;

///
///
pub struct StatusAttributeServer {
    ///
    ///
    pub inner: Arc<StdObjAttributeServer<StatusBuffer>>,
}

impl StatusAttributeServer {
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
        // Initialize the inner implementation
        let inner =
            StdObjAttributeServer::new(session, topic, task_monitor_sender, notification_channel)
                .await;

        Self {
            inner: Arc::new(inner),
        }
    }

    ///
    ///
    pub async fn set(&self, buffer: StatusBuffer) -> Result<(), Error> {
        self.inner.set(buffer).await
    }
}
