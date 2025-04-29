// mod inner;

pub mod actions;
pub mod attribute_builder;
pub mod class;
pub mod class_builder;
pub mod container;
pub mod server;
// pub mod element;
// pub mod monitor;

use attribute_builder::AttributeServerBuilder;
use class_builder::ClassBuilder;
pub use container::Container;
use panduza::task_monitor::{NamedTaskHandle, TaskHandle};
use panduza::TaskMonitor;

use crate::{engine::Engine, InstanceSettings};
use crate::{log_debug, log_error, log_trace, Actions, Logger, Notification, StateNotification};
// use class_builder::ClassBuilder;

use serde::{Deserialize, Serialize};
use std::{fmt::Display, sync::Arc};
use tokio::sync::Mutex;
use tokio::sync::{mpsc::Sender, Notify};

use async_trait::async_trait;

/// States of the main Interface FSM
///
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub enum State {
    Booting,
    Connecting,
    Initializating,
    Running,
    Warning,
    Error,
    Cleaning,
    Stopping,
    #[default]
    Undefined,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            State::Booting => write!(f, "Booting"),
            State::Connecting => write!(f, "Connecting"),
            State::Initializating => write!(f, "Initializating"),
            State::Running => write!(f, "Running"),
            State::Error => write!(f, "Error"),
            State::Warning => write!(f, "Warning"),
            State::Cleaning => write!(f, "Cleaning"),
            State::Stopping => write!(f, "Stopping"),
            State::Undefined => write!(f, "Undefined"),
        }
    }
}

///
///
///
#[derive(Clone)]
pub struct Instance {
    /// Logger for instance
    ///
    logger: Logger,

    /// Manage communications
    ///
    engine: Engine,

    /// Root topic of the instance
    ///
    topic: String,

    ///
    ///
    settings: Option<InstanceSettings>,

    /// Operations of the devices
    ///
    actions: Arc<Mutex<Box<dyn Actions>>>,

    /// State of the instance
    ///
    state: Arc<Mutex<State>>,

    /// Notifier for state change
    ///
    state_change_notifier: Arc<Notify>,

    ///
    ///
    notification_channel: Sender<Notification>,

    ///
    ///
    reset_signal: Arc<Notify>,

    ///
    ///
    task_monitor: TaskMonitor,
}

impl Instance {
    //
    // reactor

    /// Create a new instance of the Device
    ///
    pub fn new(
        engine: Engine,
        name: String,
        actions: Box<dyn Actions>,
        settings: Option<InstanceSettings>,
        notification_channel: Sender<Notification>,
    ) -> Instance {
        //
        //
        let (task_monitor, mut task_monitor_event_receiver) =
            TaskMonitor::new(format!("INSTANCE/{}", name));

        //
        let logger = Logger::new_for_instance(name.clone());

        let state = Arc::new(Mutex::new(State::Booting));
        let state_change_notifier = Arc::new(Notify::new());
        let topic = format!("{}/{}", engine.root_topic(), name);

        //
        let logger_2 = logger.clone();
        let state_2 = state.clone();
        let state_change_notifier_2 = state_change_notifier.clone();
        let notification_channel_2 = notification_channel.clone();
        let topic_2 = topic.clone();

        //
        //
        tokio::spawn(async move {
            loop {
                let event_recv = task_monitor_event_receiver.recv().await;
                println!("TASK MONITOR EVENT {:?}", event_recv);
                match event_recv {
                    Some(event) => match event {
                        panduza::task_monitor::Event::TaskMonitorError(_) => todo!(),

                        panduza::task_monitor::Event::TaskCreated(event_body) => {
                            log_trace!(logger_2, "Task {:?} started", event_body.task_name);
                        }
                        panduza::task_monitor::Event::TaskStopProperly(event_body) => {
                            log_trace!(
                                logger_2,
                                "Task {:?} stopped properly",
                                event_body.task_name
                            );
                        }

                        panduza::task_monitor::Event::TaskStopWithPain(event_body) => {
                            //
                            log_error!(
                                logger_2,
                                "Error on task {:?} - {:?}",
                                event_body.task_name,
                                event_body.error_message
                            );

                            //
                            *state_2.lock().await = State::Error;

                            notification_channel_2
                                .send(StateNotification::new(topic_2.clone(), State::Error).into())
                                .await
                                .unwrap();

                            // Notify FSM
                            state_change_notifier_2.notify_one();
                        }
                        panduza::task_monitor::Event::TaskPanicOMG(event_body) => {
                            //
                            log_error!(
                                logger_2,
                                "Panic on task {:?} - {:?}",
                                event_body.task_name,
                                event_body.error_message
                            );

                            //
                            *state_2.lock().await = State::Error;

                            notification_channel_2
                                .send(StateNotification::new(topic_2.clone(), State::Error).into())
                                .await
                                .unwrap();

                            // Notify FSM
                            state_change_notifier_2.notify_one();
                        }

                        _ => {}
                    },
                    None => {
                        // log_warn!(logger, "TaskMonitor pipe closed");
                        // Handle the error as needed
                        break;
                    }
                }
            }
        });

        // Create the object
        Instance {
            logger: logger,
            engine: engine.clone(),
            topic: topic,
            settings: settings,
            actions: Arc::new(Mutex::new(actions)),
            state: state,
            state_change_notifier: state_change_notifier,
            notification_channel: notification_channel,
            reset_signal: Arc::new(Notify::new()),
            task_monitor: task_monitor,
        }
    }

    ///
    ///
    pub fn task_monitor_sender(&self) -> Sender<NamedTaskHandle> {
        self.task_monitor.handle_sender()
    }

    ///
    /// Set the plugin name inside the logger
    ///
    pub fn set_plugin<A: Into<String>>(&mut self, text: A) {
        self.logger.set_plugin(text);
    }

    // /// Simple getter for Reactor
    // ///
    // pub fn reactor(&self) -> &Engine {
    //     &self.reactor
    // }

    /// Run the FSM of the device
    ///
    pub async fn run_fsm(&mut self) {
        //
        // First start by booting the device to give him a connection with the info_pack
        // and allow the InfoDevice to send device information on MQTT
        self.move_to_state(State::Booting).await;

        //
        // Start the main loop of the device
        // TODO => Maybe we should give a way to stop properly this task instead of canceling the task brutally
        loop {
            self.state_change_notifier.notified().await;

            // Helper log
            let stateee = self.state.lock().await.clone();
            self.logger.debug(format!("FSM State {}", stateee));

            // Perform state task
            match stateee {
                State::Booting => {
                    // if let Some(mut info_pack) = self.info_pack.clone() {
                    //     self.logger.debug("FSM try to add_deivce in info pack");
                    //     self.info_dyn_dev_status = Some(info_pack.add_device(self.name()).await);
                    //     self.logger.debug("FSM finish info pack");
                    // } else {
                    //     self.logger.debug("FSM NO INFO PACK !");
                    // }
                    self.move_to_state(State::Initializating).await;
                }
                State::Connecting => {} // wait for reactor signal
                State::Initializating => {
                    //
                    // Try to mount the device
                    let mount_result = self.actions.lock().await.mount(self.clone()).await;
                    //
                    // Manage mount result
                    match mount_result {
                        Ok(_) => {
                            self.logger.debug("FSM Mount Success ");
                            self.move_to_state(State::Running).await;
                        }
                        Err(e) => {
                            log_error!(self.logger, "Instance Mount Failure '{:?}'", e);
                            self.move_to_state(State::Error).await;
                        }
                    }
                }
                State::Running => {} // do nothing, watch for inner tasks
                State::Error => {
                    self.task_monitor.cancel_all_monitored_tasks().await;
                    //
                    // Wait before reboot
                    self.actions
                        .lock()
                        .await
                        .wait_reboot_event(self.clone())
                        .await;
                    self.logger.info("try to reboot");
                    self.move_to_state(State::Initializating).await;
                }
                State::Warning => {}
                State::Cleaning => {}
                State::Stopping => {}
                State::Undefined => {}
            }
        }

        // Ok(())
    }

    /// Clone settings of the device
    ///
    pub async fn settings(&self) -> Option<InstanceSettings> {
        self.settings.clone()
    }

    pub fn name(&self) -> String {
        match self.topic.split('/').last() {
            Some(value) => value.to_string(),
            None => "noname".to_string(),
        }
    }

    pub async fn go_error(&mut self) {
        // println!("GO ERROR");
        self.move_to_state(State::Error).await;
    }

    ///
    /// Function to change the current state of the device FSM
    ///
    pub async fn move_to_state(&mut self, new_state: State) {
        // Set the new state
        *self.state.lock().await = new_state.clone();

        // Alert monitoring device "_"
        self.notification_channel
            .send(StateNotification::new(self.topic.clone(), new_state.clone()).into())
            .await
            .unwrap();

        // Notify FSM
        self.state_change_notifier.notify_one();
    }
}

#[async_trait]
impl Container for Instance {
    /// Get for the container logger
    ///
    fn logger(&self) -> &Logger {
        &self.logger
    }

    /// Override
    ///
    fn reset_signal(&self) -> Arc<Notify> {
        self.reset_signal.clone()
    }

    /// Override
    ///
    fn trigger_reset_signal(&self) {
        self.reset_signal.notify_waiters();
    }

    /// Override
    ///
    fn create_class<N: Into<String>>(&mut self, name: N) -> ClassBuilder {
        ClassBuilder::new(
            None,
            self.clone(),
            // self.info_dyn_dev_status.clone(),
            format!("{}/{}", self.topic, name.into()),
            self.notification_channel.clone(),
        )
    }

    /// Override
    ///
    fn create_attribute<N: Into<String>>(&mut self, name: N) -> AttributeServerBuilder {
        AttributeServerBuilder::new(
            self.engine.clone(),
            None,
            self.notification_channel.clone(),
            self.task_monitor_sender().clone(),
        )
        .with_topic(format!("{}/{}", self.topic, name.into()))
    }

    /// Override
    ///
    async fn monitor_task(&self, name: String, task_handle: TaskHandle) {
        self.task_monitor_sender()
            .send((name, task_handle))
            .await
            .unwrap();
    }
}
