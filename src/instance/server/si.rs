use crate::Error;
use crate::Logger;
use bytes::Bytes;
use panduza::fbs::number::NumberBuffer;
use panduza::pubsub::Publisher;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Notify;

#[derive(Default, Debug)]
struct SiDataPack {
    /// Queue of value (need to be poped)
    ///
    queue: Vec<NumberBuffer>,

    ///
    ///
    update_notifier: Arc<Notify>,
}

impl SiDataPack {
    ///
    ///
    pub fn push(&mut self, v: NumberBuffer) {
        self.queue.push(v);
        self.update_notifier.notify_waiters();
    }

    ///
    ///
    pub fn pop(&mut self) -> Option<NumberBuffer> {
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
pub struct SiAttributeServer {
    /// Local logger
    ///
    logger: Logger,

    ///
    ///
    att_publisher: Publisher,

    /// Inner server implementation
    ///
    pack: Arc<Mutex<SiDataPack>>,

    ///
    ///
    update_notifier: Arc<Notify>,

    ///
    ///
    unit: String,

    ///
    ///
    min: f64,

    ///
    ///
    max: f64,

    ///
    ///
    decimals: usize,
}

impl SiAttributeServer {
    /// Logger getter
    ///
    pub fn logger(&self) -> &Logger {
        &self.logger
    }

    ///
    ///
    pub fn r#type() -> String {
        "string".to_string()
    }

    ///
    ///
    pub fn new<N: Into<String>>(
        topic: String,
        mut cmd_receiver: Receiver<Bytes>,
        att_publisher: Publisher,
        unit: N,
        min: f64,
        max: f64,
        decimals: usize,
    ) -> Self {
        //
        //
        let pack = Arc::new(Mutex::new(SiDataPack::default()));

        //
        // Subscribe then check for incomming messages
        let pack_2 = pack.clone();
        tokio::spawn(async move {
            loop {
                let message = cmd_receiver.recv().await;
                match message {
                    Some(data) => {
                        // Push into pack
                        pack_2
                            .lock()
                            .unwrap()
                            .push(NumberBuffer::from_raw_data(data));
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
            update_notifier: n.into(),
            unit: unit.into(),
            min: min,
            max: max,
            decimals: decimals,
        }
    }

    /// Set the value of the attribute
    ///
    pub async fn set(&self, value: NumberBuffer) -> Result<(), Error> {
        // Wrap value into payload
        let pyl = value.raw_data();

        // Send the command
        self.att_publisher.publish(pyl).await.unwrap();
        Ok(())
    }

    /// Get the value of the attribute
    /// If None, the first value is not yet received
    ///
    pub async fn pop(&mut self) -> Option<NumberBuffer> {
        self.pack.lock().unwrap().pop()
    }

    ///
    ///
    pub async fn wait_for_commands(&self) {
        self.update_notifier.notified().await;
    }
}
