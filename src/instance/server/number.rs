use crate::AlertNotification;
use crate::Error;
use crate::Logger;
use crate::Notification;
use bytes::Bytes;
// use panduza::pubsub::Publisher;
use panduza::task_monitor::NamedTaskHandle;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc::Sender;
use tokio::sync::Notify;
use zenoh::handlers::FifoChannelHandler;
use zenoh::pubsub::Subscriber;
use zenoh::sample::Sample;
use zenoh::Session;

#[derive(Default, Debug)]
struct NumberDataPack {
    /// Queue of value (need to be poped)
    ///
    queue: Vec<u32>,

    ///
    ///
    update_notifier: Arc<Notify>,
}

impl NumberDataPack {
    ///
    ///
    pub fn push(&mut self, v: u32) {
        self.queue.push(v);
        self.update_notifier.notify_waiters();
    }

    ///
    ///
    pub fn pop(&mut self) -> Option<u32> {
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
pub struct NumberAttributeServer {
    /// Local logger
    ///
    logger: Logger,

    ///
    ///
    session: Session,

    ///
    ///
    cmd_receiver: Subscriber<FifoChannelHandler<Sample>>,

    ///
    ///
    update_notifier: Arc<Notify>,

    /// topic
    ///
    topic: String,

    /// query value
    ///
    current_value: Arc<Mutex<u32>>,

    /// Channel to send notifications
    ///
    notification_channel: Sender<Notification>,
}

impl NumberAttributeServer {
    /// Logger getter
    ///
    pub fn logger(&self) -> &Logger {
        &self.logger
    }

    ///
    ///
    pub fn r#type() -> String {
        "number".to_string()
    }

    ///
    ///
    pub async fn new(
        session: Session,
        topic: String,
        mut cmd_receiver: Subscriber<FifoChannelHandler<Sample>>,
        task_monitor_sender: Sender<NamedTaskHandle>,
        notification_channel: Sender<Notification>,
    ) -> Self {
        //
        //
        let pack = Arc::new(Mutex::new(NumberDataPack::default()));
        let query_value = Arc::new(Mutex::new(u32::default()));

        // create a queryable to get value at initialization
        //
        let topic_clone = topic.clone();
        let session_clone = session.clone();
        let query_value_clone = query_value.clone();
        let handle = tokio::spawn(async move {
            let queryable = session_clone
                .declare_queryable(format!("{}/att", topic_clone.clone()))
                .await
                .unwrap();

            while let Ok(query) = queryable.recv_async().await {
                let value = query_value_clone.lock().unwrap().clone(); // Clone the value
                let pyl = Bytes::from(serde_json::to_string(&value).unwrap());
                query
                    .reply(format!("{}/att", topic_clone.clone()), pyl)
                    .await
                    .unwrap();
            }
            Ok(())
        });

        task_monitor_sender
            .send((format!("{}/server/number", &topic), handle))
            .await
            .unwrap();

        //
        //
        let n = pack.lock().unwrap().update_notifier();
        Self {
            logger: Logger::new_for_attribute_from_topic(topic.clone()),
            session: session,
            cmd_receiver: cmd_receiver,
            update_notifier: n,
            topic: topic,
            current_value: query_value,
            notification_channel: notification_channel,
        }
    }

    /// Set the value of the attribute
    ///
    pub async fn set(&self, value: u32) -> Result<(), Error> {
        // update the current queriable value
        *self.current_value.lock().unwrap() = value.clone();

        // Wrap value into payload
        let pyl = Bytes::from(serde_json::to_string(&value).unwrap());

        // Send the command
        self.session
            .put(format!("{}/att", self.topic.clone()), pyl.clone())
            .await
            .unwrap();
        Ok(())
    }

    ///
    ///
    pub async fn wait_for_commands(&self) -> Result<u32, Error> {
        let received = self.cmd_receiver.recv_async().await.unwrap();
        let value: u32 = received.payload().try_to_string().unwrap().parse().unwrap();
        Ok(value)
    }

    ///
    ///
    pub async fn trigger_alert<T: Into<String>>(&self, message: T) {
        let notification =
            Notification::Alert(AlertNotification::new(self.topic.clone(), message.into()));
        self.notification_channel.send(notification).await.unwrap();
    }
}
