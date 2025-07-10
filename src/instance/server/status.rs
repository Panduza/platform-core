use crate::instance::server::std_obj::StdObjAttributeServer;
use crate::Error;
use crate::Logger;
use crate::Notification;
use panduza::fbs::status_buffer::StatusBufferBuilder;
use panduza::fbs::InstanceStatusBuffer;
use panduza::fbs::PzaBufferBuilder;
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
    pub async fn set(&self, values: Vec<InstanceStatusBuffer>) -> Result<(), Error> {
        let buffer = StatusBufferBuilder::default()
            .with_instance_status_list(values)
            .with_source(0)
            .with_random_sequence()
            .build()
            .expect("Failed to build StatusBuffer");
        self.inner.set(buffer).await
    }
}
