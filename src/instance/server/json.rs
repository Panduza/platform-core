use crate::AlertNotification;
use crate::Error;
use crate::Logger;
use crate::Notification;
use bytes::Bytes;
// use panduza::pubsub::Publisher;
use panduza::task_monitor::NamedTaskHandle;

use serde_json::Value as JsonValue;

use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc::Sender;
// use tokio::sync::Notify;
use zenoh::handlers::FifoChannelHandler;
use zenoh::pubsub::Subscriber;
use zenoh::sample::Sample;
use zenoh::Session;

// #[derive(Default, Debug)]
// struct JsonDataPack {
//     ///
//     ///
//     update_notifier: Arc<Notify>,
// }

// impl JsonDataPack {
//     ///
//     ///
//     pub fn update_notifier(&self) -> Arc<Notify> {
//         self.update_notifier.clone()
//     }
// }

///
///
pub struct JsonAttributeServer {
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
    // update_notifier: Arc<Notify>,

    /// topic
    ///
    topic: String,

    /// query value
    ///
    current_value: Arc<Mutex<JsonValue>>,

    /// Channel to send notifications
    ///
    notification_channel: Sender<Notification>,
}

impl JsonAttributeServer {
    /// Logger getter
    ///
    pub fn logger(&self) -> &Logger {
        &self.logger
    }

    ///
    ///
    pub fn r#type() -> String {
        "json".to_string()
    }

    ///
    ///
    pub async fn new(
        session: Session,
        topic: String,
        cmd_receiver: Subscriber<FifoChannelHandler<Sample>>,
        task_monitor_sender: Sender<NamedTaskHandle>,
        notification_channel: Sender<Notification>,
    ) -> Self {
        //
        //
        // let pack = Arc::new(Mutex::new(JsonDataPack::default()));
        let query_value = Arc::new(Mutex::new(JsonValue::Null));

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
            .try_send((format!("{}/server/json", &topic), handle))
            .unwrap();

        //
        //
        // let n = pack.lock().unwrap().update_notifier();
        Self {
            logger: Logger::new_for_attribute_from_topic(topic.clone()),
            session: session,
            cmd_receiver: cmd_receiver,
            // update_notifier: n,
            topic: topic,
            current_value: query_value,
            notification_channel: notification_channel,
        }
    }

    /// Set the value of the attribute
    ///
    pub async fn set(&self, value: JsonValue) -> Result<(), Error> {
        // update the current queriable value
        *self.current_value.lock().unwrap() = value.clone();

        // Wrap value into payload
        let pyl = Bytes::from(serde_json::to_string(&value).unwrap());

        self.session
            .put(format!("{}/att", self.topic.clone()), pyl.clone())
            .await
            .unwrap();

        Ok(())
    }

    ///
    ///
    pub async fn wait_for_commands(&self) -> Result<JsonValue, Error> {
        let received = self.cmd_receiver.recv_async().await.unwrap();
        let value: JsonValue = received.payload().try_to_string().unwrap().parse().unwrap();
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
