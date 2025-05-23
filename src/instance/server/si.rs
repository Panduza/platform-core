use crate::log_trace;
use crate::Error;
use crate::Logger;
use bytes::Bytes;
use panduza::fbs::number::NumberBuffer;
use panduza::pubsub::Publisher;
use panduza::task_monitor::NamedTaskHandle;
use std::str;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
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
        "si".to_string()
    }

    ///
    ///
    pub async fn new<N: Into<String>>(
        topic: String,
        mut cmd_receiver: Receiver<Bytes>,
        att_publisher: Publisher,
        unit: N,
        min: f64,
        max: f64,
        decimals: usize,
        task_monitor_sender: Sender<NamedTaskHandle>,
    ) -> Self {
        //
        //
        let pack = Arc::new(Mutex::new(SiDataPack::default()));

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
                            .push(NumberBuffer::from_raw_data(data));
                    }
                    None => todo!(),
                }
            }
        });

        task_monitor_sender
            .send((format!("SERVER/SI >> {}", &topic), handle))
            .await
            .unwrap();

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

        //
        // TRACE
        {
            let debug_conversion = str::from_utf8(&pyl);
            if let Ok(str_data) = debug_conversion {
                log_trace!(
                    self.logger,
                    "SiAttributeServer::publish({} - {:?})",
                    str_data,
                    &pyl.to_vec()
                );
            } else {
                log_trace!(
                    self.logger,
                    "SiAttributeServer::publish({:?} )",
                    &pyl.to_vec()
                );
            }
        }

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
