use crate::AlertNotification;
use crate::Error;
use crate::Logger;
use crate::Notification;
use bytes::Bytes;
use panduza::pubsub::Publisher;
use panduza::task_monitor::NamedTaskHandle;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::Notify;

#[derive(Default, Debug)]
struct BooleanDataPack {
    // /// Last value received
    // ///
    // last: Option<bool>,
    /// Queue of value (need to be poped)
    ///
    queue: Vec<bool>,

    ///
    ///
    update_notifier: Arc<Notify>,
}

impl BooleanDataPack {
    ///
    ///
    pub fn push(&mut self, v: bool) {
        self.queue.push(v);
        self.update_notifier.notify_waiters();
    }

    ///
    ///
    pub fn pop(&mut self) -> Option<bool> {
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
#[derive(Clone)]
pub struct BooleanAttributeServer {
    /// Local logger
    ///
    logger: Logger,

    topic: String,

    ///
    ///
    att_publisher: Publisher,

    /// Inner server implementation
    ///
    pack: Arc<Mutex<BooleanDataPack>>,

    ///
    ///
    update_notifier: Arc<Notify>,

    /// Channel to send notifications
    ///
    notification_channel: Sender<Notification>,
}

impl BooleanAttributeServer {
    /// Logger getter
    ///
    pub fn logger(&self) -> &Logger {
        &self.logger
    }

    ///
    ///
    pub fn r#type() -> String {
        "boolean".to_string()
    }

    ///
    ///
    pub async fn new(
        topic: String,
        mut cmd_receiver: Receiver<Bytes>,
        att_publisher: Publisher,
        task_monitor_sender: Sender<NamedTaskHandle>,
        notification_channel: Sender<Notification>,
    ) -> Self {
        //
        //
        let pack = Arc::new(Mutex::new(BooleanDataPack::default()));

        //
        // Subscribe then check for incomming messages
        let pack_2 = pack.clone();
        let handle = tokio::spawn(async move {
            loop {
                let message = cmd_receiver.recv().await;
                match message {
                    Some(data) => {
                        // Deserialize
                        let value: bool = serde_json::from_slice(&data).unwrap();
                        // Push into pack
                        pack_2.lock().unwrap().push(value);
                    }
                    None => todo!(),
                }
            }
        });

        task_monitor_sender
            .send((format!("{}/server/boolean", &topic), handle))
            .await
            .unwrap();

        //
        //
        let n = pack.lock().unwrap().update_notifier();
        Self {
            logger: Logger::new_for_attribute_from_topic(topic.clone()),
            topic: topic.clone(),
            att_publisher: att_publisher,
            pack: pack,
            update_notifier: n,
            notification_channel: notification_channel,
        }
    }

    /// Set the value of the attribute
    ///
    pub async fn set(&self, value: bool) -> Result<(), Error> {
        // Wrap value into payload
        let pyl = Bytes::from(serde_json::to_string(&value).unwrap());

        // Send the command
        self.att_publisher.publish(pyl).await.unwrap();
        Ok(())
    }

    /// Get the value of the attribute
    /// If None, the first value is not yet received
    ///
    pub async fn pop(&mut self) -> Option<bool> {
        self.pack.lock().unwrap().pop()
    }

    ///
    ///
    pub async fn wait_for_commands(&self) {
        self.update_notifier.notified().await;
    }

    ///
    ///
    pub async fn trigger_alert<T: Into<String>>(&self, message: T) {
        let notification =
            Notification::Alert(AlertNotification::new(self.topic.clone(), message.into()));
        self.notification_channel.send(notification).await.unwrap();
    }
}
