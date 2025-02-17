use crate::Error;
use crate::{instance::attribute_builder::AttributeServerBuilder, Logger};
use bytes::Bytes;
use panduza::pubsub::Publisher;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Notify;
// use tokio::sync::Mutex;

#[derive(Default, Debug)]
struct BooleanDataPack {
    /// Last value received
    ///
    last: Option<bool>,

    /// Queue of value (need to be poped)
    ///
    queue: Vec<bool>,

    /// Update notifier
    ///
    update_notifier: Arc<Notify>,
}

impl BooleanDataPack {
    ///
    ///
    pub fn push(&mut self, v: bool) {
        // if self.use_input_queue {
        self.queue.push(v);
        // }
        // self.last = Some(v);
    }

    ///
    ///
    pub fn last(&self) -> Option<bool> {
        self.last
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

    ///
    ///
    att_publisher: Publisher,

    /// Inner server implementation
    ///
    pack: Arc<Mutex<BooleanDataPack>>,
}

impl BooleanAttributeServer {
    // /// Clone as an element object
    // ///
    // pub fn clone_as_element(&self) -> Element {
    //     Element::AsBoolean(self.clone())
    // }

    ///
    ///
    pub fn r#type() -> String {
        "boolean".to_string()
    }

    ///
    ///
    pub fn new(topic: String, mut cmd_receiver: Receiver<Bytes>, att_publisher: Publisher) -> Self {
        //
        //
        let pack = Arc::new(Mutex::new(BooleanDataPack::default()));

        //
        // Subscribe then check for incomming messages
        let pack_2 = pack.clone();
        tokio::spawn(async move {
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

        Self {
            logger: Logger::new_for_attribute_from_topic(topic.clone()),
            att_publisher: att_publisher,
            pack: pack,
        }
    }

    /// Publish a command
    ///
    async fn publish(&self, value: bool) -> Result<(), Error> {
        // let value = value.into();
        // let pyl_size = value.len();

        // self.message_client
        // .publish(&self.topic_att, QoS::AtMostOnce, true, value)
        // .await
        // .map_err(|e| Error::PublishError {
        //     topic: self.topic_att.clone(),
        //     pyl_size: pyl_size,
        //     cause: e.to_string(),
        // })

        Ok(())
    }

    // ///
    // /// Get the value of the attribute
    // /// If None, the first value is not yet received
    // ///
    // pub async fn pop_cmd(&mut self) -> Option<bool> {
    //     self.inner
    //         .lock()
    //         .await
    //         .pop_cmd()
    //         .and_then(|v| Some(v.value))
    // }

    // ///
    // /// Get the value of the attribute
    // /// If None, the first value is not yet received
    // ///
    // pub async fn get_last_cmd(&self) -> Option<bool> {
    //     return self
    //         .inner
    //         .lock()
    //         .await
    //         .get_last_cmd()
    //         .and_then(|v| Some(v.value));
    // }

    // /// Set the value of the attribute
    // ///
    // pub async fn set(&self, value: bool) -> Result<(), Error> {
    //     self.inner
    //         .lock()
    //         .await
    //         .set(BooleanCodec { value: value })
    //         .await?;
    //     Ok(())
    // }
}
