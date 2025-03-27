use crate::Error;
use crate::Logger;
use bytes::Bytes;
// use panduza::pubsub::Publisher;
use panduza::task_monitor::NamedTaskHandle;

use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::Notify;
use zenoh::handlers::FifoChannel;
use zenoh::handlers::FifoChannelHandler;
use zenoh::pubsub::Subscriber;
use zenoh::sample::Sample;
use zenoh::Session;

#[derive(Default, Debug)]
struct JsonDataPack {
    /// Queue of value (need to be poped)
    ///
    queue: Vec<JsonValue>,

    ///
    ///
    update_notifier: Arc<Notify>,
}

impl JsonDataPack {
    ///
    ///
    pub fn push(&mut self, v: JsonValue) {
        self.queue.push(v);
        self.update_notifier.notify_waiters();
    }

    ///
    ///
    pub fn pop(&mut self) -> Option<JsonValue> {
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
pub struct JsonAttributeServer {
    /// Local logger
    ///
    logger: Logger,

    ///
    ///
    session: Session,

    /// Inner server implementation
    ///
    pack: Arc<Mutex<JsonDataPack>>,

    ///
    ///
    update_notifier: Arc<Notify>,

    /// topic
    ///
    topic: String,

    /// query value
    ///
    current_value: Arc<Mutex<JsonValue>>,
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
    pub fn new(
        session: Session,
        topic: String,
        mut cmd_receiver: Subscriber<FifoChannelHandler<Sample>>,
        task_monitor_sender: Sender<NamedTaskHandle>,
    ) -> Self {
        //
        //
        let pack = Arc::new(Mutex::new(JsonDataPack::default()));
        let query_value = Arc::new(Mutex::new(JsonValue::Null));

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
            loop {
                let message = cmd_receiver.recv_async().await.unwrap();

                println!("le message du JSON : {:?}", message.clone());
                // Deserialize
                let value: JsonValue = message.payload().try_to_string().unwrap().parse().unwrap();

                println!("JSON VALUE {:?}", value.clone());
                // Push into pack
                pack_2.lock().unwrap().push(value);
            }
        });
        task_monitor_sender
            .try_send((format!("{}/server/json", &topic), handle))
            .unwrap();

        //
        //
        let n = pack.lock().unwrap().update_notifier();
        Self {
            logger: Logger::new_for_attribute_from_topic(topic.clone()),
            session: session,
            pack: pack,
            update_notifier: n,
            topic: topic,
            current_value: query_value,
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

        // while let Ok(query) = queryable.recv_async().await {
        //     query
        //         .reply(format!("{}/att", self.topic.clone()), pyl.clone())
        //         .await
        //         .unwrap();
        // }

        // Send the command
        // self.session.put(self.topic.clone(), pyl).await.unwrap();
        Ok(())
    }

    /// Get the value of the attribute
    /// If None, the first value is not yet received
    ///
    pub async fn pop(&mut self) -> Option<JsonValue> {
        self.pack.lock().unwrap().pop()
    }

    ///
    ///
    pub async fn wait_for_commands(&self) {
        self.update_notifier.notified().await;
    }
}
