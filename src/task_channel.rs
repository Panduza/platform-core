use crate::Error;
use futures::future::BoxFuture;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

/// This structure contains elements for the platform runtime can create a new task
///
pub struct TaskCreationRequest<O> {
    ///
    /// Name of the task (to help debugging to make it match with task id later)
    ///
    pub name: String,
    ///
    /// What the task must do
    ///
    pub future: BoxFuture<'static, O>,
}

impl<O> TaskCreationRequest<O> {
    ///
    /// Create a new request
    ///
    pub fn new(name: String, future: BoxFuture<'static, O>) -> TaskCreationRequest<O> {
        Self {
            name: name,
            future: future,
        }
    }
}

/// Object to monitor tasks that must be spawned
///
pub struct TaskReceiver<O> {
    /// Internal receiver
    pub rx: Receiver<TaskCreationRequest<O>>,
}

impl<O> From<Receiver<TaskCreationRequest<O>>> for TaskReceiver<O> {
    fn from(rx: Receiver<TaskCreationRequest<O>>) -> Self {
        TaskReceiver { rx: rx }
    }
}

/// Object to send task to the runner
///
#[derive(Clone)]
pub struct TaskSender<O> {
    tx: Sender<TaskCreationRequest<O>>,
}

impl<O> TaskSender<O> {
    ///
    /// Load a future into the task pool
    ///
    pub fn spawn(&mut self, future: BoxFuture<'static, O>) -> Result<(), Error> {
        let r = self
            .tx
            .try_send(TaskCreationRequest::new("unamed".to_string(), future));
        match r {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => Err(Error::Spawn(e.to_string())),
        }
    }

    ///
    /// Load a future into the task pool
    ///
    pub fn spawn_with_name<N: Into<String>>(
        &mut self,
        name: N,
        future: BoxFuture<'static, O>,
    ) -> Result<(), Error> {
        let r = self
            .tx
            .try_send(TaskCreationRequest::new(name.into(), future));
        match r {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => Err(Error::Spawn(e.to_string())),
        }
    }
}

impl<O> From<Sender<TaskCreationRequest<O>>> for TaskSender<O> {
    fn from(tx: Sender<TaskCreationRequest<O>>) -> Self {
        TaskSender { tx: tx }
    }
}

/// Create the task channel
///
pub fn create_task_channel<O>(buffer: usize) -> (TaskSender<O>, TaskReceiver<O>) {
    let (tx, rx) = channel::<TaskCreationRequest<O>>(buffer);
    return (TaskSender::<O>::from(tx), TaskReceiver::<O>::from(rx));
}
