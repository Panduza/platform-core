use crate::Error;
use crate::Logger;
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
struct EnumDataPack {
    /// Queue of value (need to be poped)
    ///
    queue: Vec<String>,

    ///
    ///
    update_notifier: Arc<Notify>,
}

impl EnumDataPack {
    ///
    ///
    pub fn push(&mut self, v: String) {
        self.queue.push(v);
        self.update_notifier.notify_waiters();
    }

    ///
    ///
    pub fn pop(&mut self) -> Option<String> {
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
pub struct EnumAttributeServer {
    /// Local logger
    ///
    logger: Logger,

    topic: String,

    ///
    ///
    session: Session,

    /// Inner server implementation
    ///
    pack: Arc<Mutex<EnumDataPack>>,

    ///
    ///
    update_notifier: Arc<Notify>,

    ///
    ///
    whitelist: Vec<String>,

    /// query value
    ///
    current_value: Arc<Mutex<String>>,
}

impl EnumAttributeServer {
    /// Logger getter
    ///
    pub fn logger(&self) -> &Logger {
        &self.logger
    }

    ///
    ///
    pub fn r#type() -> String {
        "enum".to_string()
    }

    ///
    ///
    pub fn new(
        session: Session,
        topic: String,
        mut cmd_receiver: Subscriber<FifoChannelHandler<Sample>>,
        task_monitor_sender: Sender<NamedTaskHandle>,
        whitelist: Vec<String>,
    ) -> Self {
        //
        //
        let pack = Arc::new(Mutex::new(EnumDataPack::default()));
        let query_value = Arc::new(Mutex::new(String::new()));

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
                let pyl = Bytes::from(serde_json::to_string(&value).unwrap());
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
                let value = sample.payload().try_to_string().unwrap().to_string();
                let value = serde_json::from_str::<String>(&value).unwrap_or(value);
                // Push into pack
                pack_2.lock().unwrap().push(value);
            }
            Ok(())
        });
        task_monitor_sender
            .try_send((format!("{}/server/enum", &topic), handle))
            .unwrap();

        //
        //
        let n = pack.lock().unwrap().update_notifier();
        Self {
            logger: Logger::new_for_attribute_from_topic(topic.clone()),
            session: session,
            pack: pack,
            update_notifier: n,
            whitelist: whitelist,
            current_value: query_value,
            topic: topic,
        }
    }

    /// Set the value of the attribute
    ///
    pub async fn set(&self, value: String) -> Result<(), Error> {
        // update the current queriable value
        *self.current_value.lock().unwrap() = value.clone();

        // Send the command
        self.session
            .put(format!("{}/att", self.topic.clone()), Bytes::from(value))
            .await
            .unwrap();
        Ok(())
    }

    /// Get the value of the attribute
    /// If None, the first value is not yet received
    ///
    pub async fn pop(&mut self) -> Option<String> {
        self.pack.lock().unwrap().pop()
    }

    ///
    ///
    pub async fn wait_for_commands(&self) {
        self.update_notifier.notified().await;
    }
}
