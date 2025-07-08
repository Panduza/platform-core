use crate::instance::server::ro_stream::RoStreamAttributeServer;
use crate::Error;
use crate::Logger;
use crate::Notification;
use panduza::fbs::status_buffer::StatusBufferBuilder;
use panduza::fbs::InstanceStatusBuffer;
use panduza::fbs::PzaBufferBuilder;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use zenoh::Session;

///
///
pub struct StatusAttributeServer {
    pub inner: Arc<RoStreamAttributeServer>,
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
        notification_channel: Sender<Notification>,
    ) -> Self {
        let inner = RoStreamAttributeServer::new(session, topic, notification_channel).await;

        Self {
            inner: Arc::new(inner),
        }
    }

    ///
    ///
    pub async fn push(&self, values: Vec<InstanceStatusBuffer>) -> Result<(), Error> {
        let buffer = StatusBufferBuilder::default()
            .with_instance_status_list(values)
            .build()
            .expect("Failed to build StatusBuffer");
        self.inner.set(buffer).await
    }
}
