use crate::Error;
use crate::Logger;
use bytes::Bytes;
use panduza::fbs::NotificationBuffer;
use panduza::fbs::NotificationType;
// use panduza::pubsub::Publisher;
use panduza::task_monitor::NamedTaskHandle;
use panduza::PanduzaBuffer;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc::Sender;
use tokio::sync::Notify;
use zenoh::handlers::FifoChannelHandler;
use zenoh::pubsub::Subscriber;
use zenoh::sample::Sample;
use zenoh::Session;

#[derive(Default, Debug)]
struct NotificationDataPack {
    /// Queue of value (need to be poped)
    ///
    queue: Vec<NotificationBuffer>,

    ///
    ///
    update_notifier: Arc<Notify>,
}

impl NotificationDataPack {
    ///
    ///
    pub fn push(&mut self, v: NotificationBuffer) {
        self.queue.push(v);
        self.update_notifier.notify_waiters();
    }

    ///
    ///
    pub fn pop(&mut self) -> Option<NotificationBuffer> {
        if self.queue.is_empty() {
            return None;
        }
        Some(self.queue.remove(0))
    }

    ///
    ///
    pub fn update_notifier(&self) -> Arc<Notify> {
        self.update_notifier.clone()
    }
}

///
///
pub struct NotificationAttributeServer {
    /// Local logger
    ///
    logger: Logger,

    /// topic
    ///
    topic: String,

    ///
    ///
    session: Session,

    ///
    ///
    cmd_receiver: Subscriber<FifoChannelHandler<Sample>>,

    ///
    ///
    update_notifier: Arc<Notify>,
}

impl NotificationAttributeServer {
    /// Logger getter
    ///
    pub fn logger(&self) -> &Logger {
        &self.logger
    }

    ///
    ///
    pub fn r#type() -> String {
        "notification-v0".to_string()
    }

    ///
    ///
    pub async fn new(
        session: Session,
        topic: String,
        mut cmd_receiver: Subscriber<FifoChannelHandler<Sample>>,
        task_monitor_sender: Sender<NamedTaskHandle>,
    ) -> Self {
        //
        //
        let pack = Arc::new(Mutex::new(NotificationDataPack::default()));

        // create a queryable to get value at initialization
        //
        let topic_clone = topic.clone();
        let session_clone = session.clone();

        let handle = tokio::spawn(async move {
            let queryable = session_clone
                .declare_queryable(format!("{}/att", topic_clone.clone()))
                .await
                .unwrap();

            while let Ok(query) = queryable.recv_async().await {
                let value = NotificationBuffer::new()
                    .with_notification_type(NotificationType::Alert)
                    .with_notification_source("default_source".to_string())
                    .with_notification_message("".to_string())
                    .with_source(0)
                    .with_random_sequence()
                    .build()
                    .unwrap();
                let pyl = value.to_zbytes();
                query
                    .reply(format!("{}/att", topic_clone.clone()), pyl)
                    .await
                    .unwrap();
            }
            Ok(())
        });

        task_monitor_sender
            .send((format!("SERVER/NOTIFICATION >> {}", &topic), handle))
            .await
            .unwrap();

        let n = pack.lock().unwrap().update_notifier();
        Self {
            logger: Logger::new_for_attribute_from_topic(topic.clone()),
            session: session,
            topic: topic,
            cmd_receiver: cmd_receiver,
            update_notifier: n,
            // current_value: query_value,
        }
    }

    /// Set the value of the attribute
    ///
    pub async fn set(
        &self,
        r#type: NotificationType,
        source: String,
        message: String,
    ) -> Result<(), Error> {
        let buffer = NotificationBuffer::new()
            .with_notification_type(r#type)
            .with_notification_source(source)
            .with_notification_message(message)
            .with_random_sequence()
            .build()
            .map_err(|e| Error::Generic(e))?;

        // Send the command
        self.session
            .put(format!("{}/att", self.topic.clone()), buffer.to_zbytes())
            .await
            .unwrap();
        Ok(())
    }

    /// Set the buffer
    ///
    pub async fn set_buffer(&self, buffer: NotificationBuffer) -> Result<(), Error> {
        // Send the command
        self.session
            .put(format!("{}/att", self.topic.clone()), buffer.to_zbytes())
            .await
            .unwrap();
        Ok(())
    }

    ///
    ///
    pub async fn wait_for_commands(&self) -> Result<NotificationBuffer, Error> {
        let received = self.cmd_receiver.recv_async().await.unwrap();
        Ok(NotificationBuffer::build_from_zbytes(
            received.payload().clone(),
        ))
    }
}
