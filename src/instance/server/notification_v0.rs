use crate::Error;
use crate::Logger;
use bytes::Bytes;
use panduza::fbs::notification_v0::NotificationBuffer;
use panduza::fbs::notification_v0::NotificationType;
// use panduza::pubsub::Publisher;
use panduza::task_monitor::NamedTaskHandle;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc::Receiver;
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
#[derive(Clone)]
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

    /// Inner server implementation
    ///
    pack: Arc<Mutex<NotificationDataPack>>,

    ///
    ///
    update_notifier: Arc<Notify>,

    /// query value
    ///
    current_value: Arc<Mutex<NotificationBuffer>>,
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
        let query_value = Arc::new(Mutex::new(NotificationBuffer::default()));

        // create a queryable to get value at initialization
        //
        let topic_clone = topic.clone();
        let session_clone = session.clone();
        let query_value_clone = query_value.clone();
        tokio::spawn(async move {
            let queryable = session_clone
                .declare_queryable(format!("{}/att", topic_clone.clone()))
                .await
                .unwrap();

            while let Ok(query) = queryable.recv_async().await {
                let value = query_value_clone.lock().unwrap().clone(); // Clone the value
                let pyl = value.take_data();
                query
                    .reply(format!("{}/att", topic_clone.clone()), pyl)
                    .await
                    .unwrap();
            }
        });

        //
        // Subscribe then check for incomming messages
        let pack_2 = pack.clone();
        let handle = tokio::spawn(async move {
            while let Ok(sample) = cmd_receiver.recv_async().await {
                let value: Bytes = Bytes::copy_from_slice(&sample.payload().to_bytes());
                // Push into pack
                pack_2
                    .lock()
                    .unwrap()
                    .push(NotificationBuffer::from_raw_data(value));
            }
            Ok(())
        });
        task_monitor_sender
            .send((format!("SERVER/STATUS >> {}", &topic), handle))
            .await
            .unwrap();
        //
        //
        let n = pack.lock().unwrap().update_notifier();
        Self {
            logger: Logger::new_for_attribute_from_topic(topic.clone()),
            session: session,
            topic: topic,
            pack: pack,
            update_notifier: n,
            current_value: query_value,
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
        let buffer = NotificationBuffer::from_args(r#type, source, message);
        // update the current queriable value
        *self.current_value.lock().unwrap() = buffer.clone();

        // Send the command
        self.session
            .put(format!("{}/att", self.topic.clone()), buffer.take_data())
            .await
            .unwrap();
        Ok(())
    }

    /// Set the buffer
    ///
    pub async fn set_buffer(&self, buffer: NotificationBuffer) -> Result<(), Error> {
        // Send the command
        self.session
            .put(format!("{}/att", self.topic.clone()), buffer.take_data())
            .await
            .unwrap();
        Ok(())
    }

    /// Get the value of the attribute
    /// If None, the first value is not yet received
    ///
    pub async fn pop(&mut self) -> Option<NotificationBuffer> {
        self.pack.lock().unwrap().pop()
    }

    ///
    ///
    pub async fn wait_for_commands(&self) {
        self.update_notifier.notified().await;
    }
}
