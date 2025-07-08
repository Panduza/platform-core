use crate::Error;
use crate::Logger;
use panduza::fbs::InstanceStatusBuffer;
use panduza::fbs::PzaBuffer;
use panduza::fbs::StatusBuffer;
use panduza::task_monitor::NamedTaskHandle;
use panduza::PanduzaBuffer;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc::Sender;
use tokio::sync::Notify;
use zenoh::handlers::FifoChannelHandler;
use zenoh::pubsub::Subscriber;
use zenoh::sample::Sample;
use zenoh::Session;

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
pub struct StatusAttributeServer {
    /// Local logger
    ///
    logger: Logger,

    /// topic
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
    current_value: Arc<Mutex<StatusBuffer>>,
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
        session: Session,
        topic: String,
        cmd_receiver: Subscriber<FifoChannelHandler<Sample>>,
        task_monitor_sender: Sender<NamedTaskHandle>,
    ) -> Self {
        //
        //
        let pack = Arc::new(Mutex::new(StatusDataPack::default()));

        // Default initial value
        let initial_value = StatusBuffer::new()
            .with_instance_status_list(vec![])
            .with_random_sequence()
            .build()
            .unwrap();

        let current_value = Arc::new(Mutex::new(initial_value.clone()));

        // create a queryable to get value at initialization
        //
        let topic_clone = topic.clone();
        let session_clone = session.clone();
        let current_value_clone = current_value.clone();

        let handle = tokio::spawn(async move {
            let queryable = session_clone
                .declare_queryable(format!("{}/att", topic_clone.clone()))
                .await
                .unwrap();

            while let Ok(query) = queryable.recv_async().await {
                let current_val = current_value_clone.lock().unwrap().clone();
                let pyl = current_val.to_zbytes();
                query
                    .reply(format!("{}/att", topic_clone.clone()), pyl)
                    .await
                    .unwrap();
            }
            Ok(())
        });

        task_monitor_sender
            .send((format!("SERVER/STATUS >> {}", &topic), handle))
            .await
            .unwrap();

        let n = pack.lock().unwrap().update_notifier();
        Self {
            logger: Logger::new_for_attribute_from_topic(topic.clone()),
            session: session,
            topic: topic,
            cmd_receiver: cmd_receiver,
            update_notifier: n,
            current_value: current_value,
        }
    }

    /// Set the value of the attribute
    ///
    pub async fn set(&self, all_status: Vec<InstanceStatusBuffer>) -> Result<(), Error> {
        let buffer = StatusBuffer::new()
            .with_instance_status_list(all_status)
            .with_random_sequence()
            .build()
            .map_err(|e| Error::Generic(e))?;

        // update the current queriable value
        *self.current_value.lock().unwrap() = buffer.clone();

        // Send the command
        self.session
            .put(format!("{}/att", self.topic.clone()), buffer.to_zbytes())
            .await
            .unwrap();
        Ok(())
    }

    /// Set the buffer
    ///
    pub async fn set_buffer(&self, buffer: StatusBuffer) -> Result<(), Error> {
        // update the current queriable value
        *self.current_value.lock().unwrap() = buffer.clone();

        // Send the command
        self.session
            .put(format!("{}/att", self.topic.clone()), buffer.to_zbytes())
            .await
            .unwrap();
        Ok(())
    }

    ///
    ///
    pub async fn wait_for_commands(&self) -> Result<StatusBuffer, Error> {
        let received = self.cmd_receiver.recv_async().await.unwrap();
        Ok(StatusBuffer::from_zbytes(received.payload().clone()))
    }
}
