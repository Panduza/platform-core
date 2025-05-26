use crate::Error;
use crate::Logger;
use bytes::Bytes;
use panduza::fbs::sample::SampleBuffer;
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
struct SampleDataPack {
    /// Queue of value (need to be poped)
    ///
    queue: Vec<SampleBuffer>,

    ///
    ///
    update_notifier: Arc<Notify>,
}

impl SampleDataPack {
    ///
    ///
    pub fn push(&mut self, v: SampleBuffer) {
        self.queue.push(v);
        self.update_notifier.notify_waiters();
    }

    ///
    ///
    pub fn pop(&mut self) -> Option<SampleBuffer> {
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
pub struct SampleAttributeServer {
    /// Local logger
    ///
    logger: Logger,

    /// Topic
    ///
    topic: String,

    /// Session
    ///
    session: Session,

    ///
    ///
    cmd_receiver: Subscriber<FifoChannelHandler<Sample>>,

    ///
    ///
    update_notifier: Arc<Notify>,

    /// query value
    ///
    current_value: Arc<Mutex<SampleBuffer>>,
}

impl SampleAttributeServer {
    /// Logger getter
    ///
    pub fn logger(&self) -> &Logger {
        &self.logger
    }

    ///
    ///
    pub fn r#type() -> String {
        "sample".to_string()
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
        let pack = Arc::new(Mutex::new(SampleDataPack::default()));
        let query_value = Arc::new(Mutex::new(SampleBuffer::default()));

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
                let pyl = value.take_data();
                query
                    .reply(format!("{}/att", topic_clone.clone()), pyl)
                    .await
                    .unwrap();
            }
            Ok(())
        });

        task_monitor_sender
            .send((format!("{}/server/sample", &topic), handle))
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
            update_notifier: n,
            current_value: query_value,
        }
    }

    /// Set the value of the attribute
    ///
    pub async fn set(&self, values: &Vec<f32>) -> Result<(), Error> {
        // Wrap value into payload
        let pyl = SampleBuffer::from_values(values);

        // update the current queriable value
        *self.current_value.lock().unwrap() = pyl.clone();

        // Send the command
        self.session
            .put(format!("{}/att", self.topic.clone()), pyl.take_data())
            .await
            .unwrap();
        Ok(())
    }

    ///
    ///
    pub async fn wait_for_commands(&self) -> Result<SampleBuffer, Error> {
        let received = self.cmd_receiver.recv_async().await.unwrap();
        let value: SampleBuffer =
            SampleBuffer::from_raw_data(Bytes::copy_from_slice(&received.payload().to_bytes()));
        Ok(value)
    }
}
