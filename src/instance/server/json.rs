use crate::Error;
use crate::Logger;
use bytes::Bytes;
use panduza::pubsub::Publisher;
use serde_json::Value as JsonValue;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Notify;

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
    att_publisher: Publisher,

    /// Inner server implementation
    ///
    pack: Arc<Mutex<JsonDataPack>>,

    ///
    ///
    update_notifier: Arc<Notify>,
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
        "Json".to_string()
    }

    ///
    ///
    pub fn new(topic: String, mut cmd_receiver: Receiver<Bytes>, att_publisher: Publisher) -> Self {
        //
        //
        let pack = Arc::new(Mutex::new(JsonDataPack::default()));

        //
        // Subscribe then check for incomming messages
        let pack_2 = pack.clone();
        tokio::spawn(async move {
            loop {
                let message = cmd_receiver.recv().await;
                match message {
                    Some(data) => {
                        // Deserialize
                        let value: JsonValue = serde_json::from_slice(&data).unwrap();
                        // Push into pack
                        pack_2.lock().unwrap().push(value);
                    }
                    None => todo!(),
                }
            }
        });

        //
        //
        let n = pack.lock().unwrap().update_notifier();
        Self {
            logger: Logger::new_for_attribute_from_topic(topic.clone()),
            att_publisher: att_publisher,
            pack: pack,
            update_notifier: n,
        }
    }

    /// Set the value of the attribute
    ///
    pub async fn set(&self, value: JsonValue) -> Result<(), Error> {
        // Wrap value into payload
        let pyl = Bytes::from(serde_json::to_string(&value).unwrap());

        // Send the command
        self.att_publisher.publish(pyl).await.unwrap();
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
