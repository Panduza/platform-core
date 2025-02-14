pub mod notification;

use crate::{log_debug, log_warn, Logger, Notification, NotificationGroup};
use crate::{
    task_channel::create_task_channel, Factory, ProductionOrder, Reactor, TaskReceiver, TaskResult,
    TaskSender,
};
// use futures::lock::Mutex;
use futures::FutureExt;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc::channel, Notify};
use tokio::task::JoinSet;

///
///
///
static TASK_CHANNEL_SIZE: usize = 64;

///
///
///
static PROD_ORDER_CHANNEL_SIZE: usize = 64;

///
///
///
static NOTIFICATION_CHANNEL_SIZE: usize = 512;

/// Manage the execution instances
///
pub struct Runtime {
    ///
    /// Logger dedicated to runtime activity
    ///
    logger: Logger,
    ///
    ///
    factory: Factory,
    ///
    ///
    reactor: Reactor,
    ///
    /// Flag to know if we the runtime must continue its work
    keep_alive: Arc<AtomicBool>,
    ///
    /// Flag to know alert the platform, it must stop
    must_stop: Arc<AtomicBool>,
    ///
    /// Pool
    task_pool: JoinSet<TaskResult>,
    ///
    /// Sender, allow a sub function to start a task managed by this runtime
    task_sender: TaskSender<TaskResult>,
    ///
    /// Receiver, catch task request and start them inside this runtime
    task_receiver: Option<TaskReceiver<TaskResult>>,
    ///
    /// Notify when a new task has been loaded
    ///
    new_task_notifier: Arc<Notify>,

    ///
    /// Sender, allow a sub function to request a register a production order
    production_order_sender: Sender<ProductionOrder>,
    /// Sender, allow a sub function to request a register a production order
    production_order_receiver: Option<Receiver<ProductionOrder>>,

    ///
    /// Notifications that comes from devices
    /// They will help the underscore device to give informations to the user
    ///
    notifications: Arc<std::sync::Mutex<NotificationGroup>>,
    ///
    notification_sender: Sender<Notification>,
    ///
    notification_receiver: Option<Receiver<Notification>>,
}

impl Runtime {
    ///
    /// Constructor
    ///
    pub fn new(factory: Factory, reactor: Reactor) -> Self {
        let (t_tx, t_rx) = create_task_channel::<TaskResult>(TASK_CHANNEL_SIZE);
        let (po_tx, po_rx) = channel::<ProductionOrder>(PROD_ORDER_CHANNEL_SIZE);
        let (not_tx, not_rx) = channel::<Notification>(NOTIFICATION_CHANNEL_SIZE);

        Self {
            logger: Logger::new_for_runtime(),
            factory: factory,
            reactor: reactor,
            keep_alive: Arc::new(AtomicBool::new(true)),
            must_stop: Arc::new(AtomicBool::new(false)),
            task_pool: JoinSet::new(),
            task_sender: t_tx.clone(),
            task_receiver: Some(t_rx),
            new_task_notifier: Arc::new(Notify::new()),
            production_order_sender: po_tx.clone(),
            production_order_receiver: Some(po_rx),
            notifications: Arc::new(std::sync::Mutex::new(NotificationGroup::new())),
            notification_sender: not_tx.clone(),
            notification_receiver: Some(not_rx),
        }
    }

    ///
    /// Set the plugin name inside the logger
    ///
    pub fn set_plugin<A: Into<String>>(&mut self, text: A) {
        self.logger.set_plugin(text);
    }

    ///
    /// Getter for 'task_sender', need to be get before task start
    ///
    pub fn clone_task_sender(&self) -> TaskSender<TaskResult> {
        self.task_sender.clone()
    }

    ///
    /// Getter for 'production_order_sender', need to be get before task start
    ///
    pub fn clone_production_order_sender(&self) -> Sender<ProductionOrder> {
        self.production_order_sender.clone()
    }

    ///
    ///
    ///
    pub fn clone_notifications(&self) -> Arc<std::sync::Mutex<NotificationGroup>> {
        self.notifications.clone()
    }

    ///
    /// Main task of the runtime, it consume the object itself
    ///
    pub async fn task(mut self) -> TaskResult {
        //
        // Debug log
        self.logger.info("Runtime started !");

        self.reactor.start(self.task_sender.clone()).unwrap();

        //
        // Remove task receiver from self
        let mut task_receiver = self
            .task_receiver
            .take()
            .ok_or(crate::Error::InternalLogic(
                "Object 'task_receiver' is 'None'".to_string(),
            ))?;

        //
        // Remove production order receiver from self
        let mut p_order_receiver =
            self.production_order_receiver
                .take()
                .ok_or(crate::Error::InternalLogic(
                    "Object 'production_order_receiver' is 'None'".to_string(),
                ))?;

        //
        // Remove production order receiver from self
        let mut notification_receiver =
            self.notification_receiver
                .take()
                .ok_or(crate::Error::InternalLogic(
                    "Object 'notification_receiver' is 'None'".to_string(),
                ))?;

        //
        while self.keep_alive.load(Ordering::Relaxed) {
            tokio::select! {
                //
                // Manage new task creation requests
                //
                request = task_receiver.rx.recv() => {
                    match request {
                        Some(task) => {
                            // Function to effectily spawn tasks requested by the system
                            let ah = self.task_pool.spawn(task.future);
                            log_debug!(self.logger, "New task created [{:?} => {:?}]", ah.id(), task.name );
                            self.new_task_notifier.notify_waiters();
                        },
                        None => {
                            log_warn!(self.logger, "Empty Task Request Received !");
                        }
                    }
                },
                //
                //
                //
                production_order = p_order_receiver.recv() => {

                    self.logger.debug(format!( "PROD REQUEST ! [{:?}]", production_order ));

                    let name = production_order.as_ref().unwrap().name.clone();

                    // let mut production_order = ProductionOrder::new("panduza.picoha-dio", "testdevice");
                    // production_order.device_settings = json!({});
                    let (mut monitor, mut dev) =
                        self.factory
                            .produce(self.reactor.clone(), Some(self.notification_sender.clone()), production_order.unwrap());

                    dev.set_plugin(self.logger.get_plugin());

                    // let mut dddddd2 = dev.clone();
                    self.task_sender
                        .spawn_with_name(
                            format!("{}/fsm", name),
                            async move {
                                dev.run_fsm().await;
                                Ok(())
                            }
                            .boxed(),
                        )
                        .unwrap();

                    self.task_sender
                        .spawn_with_name(
                            format!("{}/monitor", name),
                            async move {
                                monitor.run().await;
                                Ok(())
                            }
                            .boxed(),
                        )
                        .unwrap();

                },
                notif = notification_receiver.recv() => {

                    // self.logger.trace(format!( "NOTIF [{:?}]", notif ));

                    self.notifications.lock().unwrap().push(notif.unwrap());
                },
                //
                // task to create monitor plugin manager notifications
                //
                continue_running = self.end_of_all_tasks() => {
                    //
                    // Manage platform end
                    if !continue_running {
                        break;
                    }
                }
            }
        }

        //
        // Debug log
        self.logger.warn("Runtime over !");

        //
        // Return ok
        Ok(())
    }

    /// Wait for all tasks to complete
    ///
    async fn end_of_all_tasks(&mut self) -> bool {
        //
        // Make tasks run
        while let Some(join_result) = self.task_pool.join_next().await {
            match join_result {
                Ok(jr) => match jr {
                    Ok(_) => {
                        log_debug!(self.logger, "Task completed successly");
                    }
                    Err(e) => {
                        self.logger.error(format!("Task end badly: {:?}", e));
                        self.task_pool.abort_all();
                    }
                },
                Err(e) => {
                    self.logger.error(format!("Task join_next error: {:?}", e));
                }
            }
        }
        //
        // Reaching here means that there is no task anymore
        log_debug!(self.logger, "All tasks completed");
        match self.must_stop.load(Ordering::Relaxed) {
            true => {
                // No task and stop request => quit this loop
                false
            }
            false => {
                // Wait for an other task to be loaded
                log_debug!(self.logger, "Wait for new tasks");
                self.new_task_notifier.notified().await;
                true
            }
        }
    }
}
