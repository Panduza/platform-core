use crate::instance::server::std_obj::StdObjAttributeServer;
use crate::Error;
use crate::Logger;
use crate::Notification;
use panduza::fbs::PzaBufferBuilder;
use panduza::fbs::StructureBuffer;
use panduza::task_monitor::NamedTaskHandle;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use zenoh::Session;

///
///
pub struct StructureAttributeServer {
    ///
    ///
    pub inner: Arc<StdObjAttributeServer<StructureBuffer>>,
}

impl StructureAttributeServer {
    // ------------------------------------------------------------------------

    /// Logger getter
    ///
    pub fn logger(&self) -> &Logger {
        self.inner.logger()
    }

    // ------------------------------------------------------------------------

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

    // ------------------------------------------------------------------------

    ///
    ///
    pub async fn empty_initial_set(&self) -> Result<(), Error> {
        let buffer = StructureBuffer::builder()
            .with_source(0)
            .with_random_sequence()
            .build()
            .expect("Failed to build StructureBuffer");
        self.inner.set(buffer).await
    }

    // ------------------------------------------------------------------------

    ///
    ///
    pub async fn set(&self, buffer: StructureBuffer) -> Result<(), Error> {
        self.inner.set(buffer).await
    }

    // ------------------------------------------------------------------------
}
