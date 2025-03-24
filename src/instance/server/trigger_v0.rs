use crate::Error;
use crate::Logger;
use bytes::Bytes;
use panduza::fbs::trigger_v0::TriggerBuffer;
use panduza::pubsub::Publisher;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Notify;

#[derive(Default, Debug)]
struct TriggerDataPack {
    /// Queue of value (need to be poped)
    ///
    queue: Vec<TriggerBuffer>,

    ///
    ///
    update_notifier: Arc<Notify>,
}

impl TriggerDataPack {
    ///
    ///
    pub fn push(&mut self, v: TriggerBuffer) {
        self.queue.push(v);
        self.update_notifier.notify_waiters();
    }

    ///
    ///
    pub fn pop(&mut self) -> Option<TriggerBuffer> {
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
pub struct TriggerAttributeServer {
    /// Local logger
    ///
    logger: Logger,

    ///
    ///
    att_publisher: Publisher,

    /// Inner server implementation
    ///
    pack: Arc<Mutex<TriggerDataPack>>,

    ///
    ///
    update_notifier: Arc<Notify>,
}

impl TriggerAttributeServer {
    /// Logger getter
    ///
    pub fn logger(&self) -> &Logger {
        &self.logger
    }

    ///
    ///
    pub fn r#type() -> String {
        "trigger-v0".to_string()
    }

    ///
    ///
    pub fn new(topic: String, mut cmd_receiver: Receiver<Bytes>, att_publisher: Publisher) -> Self {
        //
        //
        let pack = Arc::new(Mutex::new(TriggerDataPack::default()));

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
                            .push(TriggerBuffer::from_raw_data(data));
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
    pub async fn set(&self, refresh: f32) -> Result<(), Error> {
        // Wrap value into payload
        let pyl = TriggerBuffer::from_values(refresh, 0, None, None);

        // Send the command
        self.att_publisher.publish(pyl.take_data()).await.unwrap();
        Ok(())
    }

    /// Get the value of the attribute
    /// If None, the first value is not yet received
    ///
    pub async fn pop(&mut self) -> Option<TriggerBuffer> {
        self.pack.lock().unwrap().pop()
    }

    ///
    ///
    pub async fn wait_for_commands(&self) {
        self.update_notifier.notified().await;
    }
}
