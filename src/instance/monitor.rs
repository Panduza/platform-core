use super::Instance;
use crate::task_channel::create_task_channel;
use crate::{log_debug, log_warn, DriverOperations, Reactor, TaskReceiver};
use crate::{Error, Notification, ProductionOrder};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::Notify;
use tokio::time::sleep;
use tokio::{sync::Mutex, task::JoinSet};

/// Result for task spawned by the device subtasks
///
pub type DeviceTaskResult = Result<(), Error>;

/// Object to manage device subtasks
/// It is important to check when a task has failed
///
pub struct InstanceMonitor {
    /// To allow the communication with the state machine
    ///
    device: Instance,

    subtask_pool: JoinSet<DeviceTaskResult>,
    subtask_receiver: Arc<Mutex<TaskReceiver<DeviceTaskResult>>>,

    subtask_pool_not_empty_notifier: Arc<Notify>,
}

impl InstanceMonitor {
    ///
    /// Constructor
    pub fn new(
        reactor: Reactor,
        r_notifier: Option<Sender<Notification>>,
        operations: Box<dyn DriverOperations>,
        production_order: ProductionOrder,
    ) -> (InstanceMonitor, Instance) {
        //
        // Move in data and consume production order
        let name = production_order.name;
        let settings = production_order.settings;
        //
        // Create the task channel between the device and its monitoring object
        let (task_tx, task_rx) = create_task_channel::<DeviceTaskResult>(50);
        //
        // Create the device object
        let device = Instance::new(
            reactor.clone(),
            r_notifier,
            task_tx,
            name,
            operations,
            settings,
        );
        //
        // Create the monitoring object
        let monitor = InstanceMonitor {
            device: device.clone(),
            subtask_pool: JoinSet::new(),
            subtask_receiver: Arc::new(Mutex::new(task_rx)),
            subtask_pool_not_empty_notifier: Arc::new(Notify::new()),
        };
        //
        // Ok
        (monitor, device)
    }

    pub async fn run(&mut self) {
        let subtask_receiver_clone = self.subtask_receiver.clone();
        let mut subtask_receiver_clone_lock = subtask_receiver_clone.lock().await;
        let subtask_pool_not_empty_notifier_clone = self.subtask_pool_not_empty_notifier.clone();
        loop {
            tokio::select! {
                //
                // Manage new task creation requests
                //
                request = subtask_receiver_clone_lock.rx.recv() => {
                    match request {
                        Some(task) => {
                            // Function to effectily spawn tasks requested by the system
                            let ah = self.subtask_pool.spawn(task.future);
                            log_debug!(self.device.logger, "New task created [{:?} => {:?}]", ah.id(), task.name );
                            subtask_pool_not_empty_notifier_clone.notify_waiters();
                        },
                        None => {
                            log_warn!(self.device.logger, "Empty Task Request Received !");
                        }
                    }

                },
                //
                //
                //
                _ = self.end_of_all_tasks() => {
                    // Juste alert the user, but it can be not important
                    self.device.logger.warn("No sub task anymore");

                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    /// Wait for all tasks to complete
    ///
    async fn end_of_all_tasks(&mut self) {
        //
        // Wait for some task in the pool if the pool is empty
        if self.subtask_pool.is_empty() {
            self.subtask_pool_not_empty_notifier.notified().await;
        }

        while let Some(join_result) = self.subtask_pool.join_next().await {
            // self.services.lock().await.stop_requested();

            match join_result {
                Ok(task_result) => match task_result {
                    Ok(_) => {
                        println!("Task completed");
                    }
                    Err(e) => {
                        //
                        // Debug log when the sub task crash
                        self.device
                            .logger
                            .error(format!("Instance sub task crash: {}", e));

                        self.subtask_pool.abort_all();

                        self.device.go_error().await;
                    }
                },
                Err(e) => {
                    println!("Join failed: {:?}", e);
                }
            }
        }
    }
}
