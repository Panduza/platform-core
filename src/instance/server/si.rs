use crate::log_trace;
use crate::Error;
use crate::Logger;
use bytes::Bytes;
use panduza::fbs::number::NumberBuffer;
use panduza::task_monitor::NamedTaskHandle;
use std::str;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::Notify;
use zenoh::handlers::FifoChannelHandler;
use zenoh::pubsub::Subscriber;
use zenoh::sample::Sample;
use zenoh::session::Session;

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
pub struct SiAttributeServer {
    /// Local logger
    ///
    logger: Logger,

    /// Topic
    ///
    topic: String,

    ///
    ///
    session: Session,

    ///
    ///
    cmd_receiver: Subscriber<FifoChannelHandler<Sample>>,

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

    /// query value
    ///
    current_value: Arc<Mutex<NumberBuffer>>,
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
        session: Session,
        topic: String,
        mut cmd_receiver: Subscriber<FifoChannelHandler<Sample>>,
        unit: N,
        min: f64,
        max: f64,
        decimals: usize,
        task_monitor_sender: Sender<NamedTaskHandle>,
    ) -> Self {
        //
        //
        let pack = Arc::new(Mutex::new(SiDataPack::default()));
        let query_value = Arc::new(Mutex::new(NumberBuffer::default()));

        // create a queryable to get value at initialization
        //
        let topic_clone = topic.clone();
        let session_clone = session.clone();
        let query_value_clone = query_value.clone();
        let handle = tokio::spawn(async move {
            let queryable = session_clone
                .declare_queryable(format!("{}/att", topic_clone.clone()))
                .await
                .unwrap();

            while let Ok(query) = queryable.recv_async().await {
                let value = query_value_clone.lock().unwrap().clone(); // Clone the value
                let pyl = value.raw_data();
                query
                    .reply(format!("{}/att", topic_clone.clone()), pyl)
                    .await
                    .unwrap();
            }
            Ok(())
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
            session: session,
            topic: topic,
            cmd_receiver: cmd_receiver,
            update_notifier: n.into(),
            unit: unit.into(),
            min: min,
            max: max,
            decimals: decimals,
            current_value: query_value,
        }
    }

    /// Set the value of the attribute
    ///
    pub async fn set(&self, value: NumberBuffer) -> Result<(), Error> {
        // update the current queriable value
        *self.current_value.lock().unwrap() = value.clone();

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
        self.session
            .put(format!("{}/att", self.topic.clone()), pyl)
            .await
            .unwrap();
        Ok(())
    }

    ///
    ///
    pub async fn wait_for_commands(&self) -> Result<NumberBuffer, Error> {
        let received = self.cmd_receiver.recv_async().await.unwrap();
        let value: NumberBuffer =
            NumberBuffer::from_raw_data(Bytes::copy_from_slice(&received.payload().to_bytes()));
        Ok(value)
    }
}
