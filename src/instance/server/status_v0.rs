use crate::Error;
use crate::Logger;
use bytes::Bytes;
use panduza::fbs::status_v0::InstanceStatusBuffer;
use panduza::fbs::status_v0::StatusBuffer;
use panduza::pubsub::Publisher;
use panduza::task_monitor::NamedTaskHandle;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::Notify;

#[derive(Default, Debug)]
struct StatusDataPack {
    /// Queue of value (need to be poped)
    ///
    queue: Vec<StatusBuffer>,

    ///
    ///
    update_notifier: Arc<Notify>,
}

impl StatusDataPack {
    ///
    ///
    pub fn push(&mut self, v: StatusBuffer) {
        self.queue.push(v);
        self.update_notifier.notify_waiters();
    }

    ///
    ///
    pub fn pop(&mut self) -> Option<StatusBuffer> {
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
pub struct StatusAttributeServer {
    /// Local logger
    ///
    logger: Logger,

    ///
    ///
    att_publisher: Publisher,

    /// Inner server implementation
    ///
    pack: Arc<Mutex<StatusDataPack>>,

    ///
    ///
    update_notifier: Arc<Notify>,
}

impl StatusAttributeServer {
    /// Logger getter
    ///
    pub fn logger(&self) -> &Logger {
        &self.logger
    }

    ///
    ///
    pub fn r#type() -> String {
        "status-v0".to_string()
    }

    ///
    ///
    pub async fn new(
        topic: String,
        mut cmd_receiver: Receiver<Bytes>,
        att_publisher: Publisher,
        task_monitor_sender: Sender<NamedTaskHandle>,
    ) -> Self {
        //
        //
        let pack = Arc::new(Mutex::new(StatusDataPack::default()));

        //
        // Subscribe then check for incomming messages
        let pack_2 = pack.clone();
        let handle = tokio::spawn(async move {
            loop {
                let message = cmd_receiver.recv().await;
                match message {
                    Some(data) => {
                        // Push into pack
                        pack_2
                            .lock()
                            .unwrap()
                            .push(StatusBuffer::from_raw_data(data));
                    }
                    None => todo!(),
                }
            }
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
            att_publisher: att_publisher,
            pack: pack,
            update_notifier: n,
        }
    }

    /// Set the value of the attribute
    ///
    pub async fn set(&self, all_status: Vec<InstanceStatusBuffer>) -> Result<(), Error> {
        // Wrap value into payload
        let pyl = StatusBuffer::from_args(all_status);

        // Send the command
        self.att_publisher.publish(pyl.take_data()).await.unwrap();
        Ok(())
    }

    /// Get the value of the attribute
    /// If None, the first value is not yet received
    ///
    pub async fn pop(&mut self) -> Option<StatusBuffer> {
        self.pack.lock().unwrap().pop()
    }

    ///
    ///
    pub async fn wait_for_commands(&self) {
        self.update_notifier.notified().await;
    }
}
