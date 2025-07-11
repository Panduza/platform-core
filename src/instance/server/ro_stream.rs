use crate::AlertNotification;
use crate::Error;
use crate::Logger;
use crate::Notification;
use panduza::fbs::PzaBuffer;
use tokio::sync::mpsc::Sender;
use zenoh::Session;

/// Generic attribute implementation that can work with any buffer type that implements PzaBuffer
// #[derive(Clone)]
pub struct RoStreamAttributeServer {
    /// Local logger
    logger: Logger,

    /// Global Session
    session: Session,

    /// Attribute topic
    att_topic: String,

    /// Topic of the attribute
    topic: String,

    /// Channel to send notifications
    notification_channel: Sender<Notification>,
}

impl RoStreamAttributeServer {
    /// Logger getter
    ///
    pub fn logger(&self) -> &Logger {
        &self.logger
    }

    ///
    ///
    pub async fn new(
        session: Session,
        topic: String,
        notification_channel: Sender<Notification>,
    ) -> Self {
        //
        let att_topic = format!("{}/att", &topic);

        //
        Self {
            logger: Logger::new_for_attribute_from_topic(topic.clone()),
            session: session,
            att_topic: att_topic,
            topic: topic,
            notification_channel: notification_channel,
        }
    }

    ///
    ///
    pub async fn push<B: PzaBuffer>(&self, buffer: B) -> Result<(), Error> {
        // Send the command
        self.session
            .put(&self.att_topic, buffer.to_zbytes())
            .await
            .unwrap();
        Ok(())
    }

    ///
    ///
    pub async fn trigger_alert<T: Into<String>>(&self, message: T) {
        let notification =
            Notification::Alert(AlertNotification::new(self.topic.clone(), message.into()));
        self.notification_channel.send(notification).await.unwrap();
    }
}
