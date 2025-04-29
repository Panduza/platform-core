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
use panduza::task_monitor::NamedTaskHandle;
use panduza::TaskMonitor;

use crate::{engine::Engine, InstanceSettings};
use crate::{log_error, Actions, Logger, Notification};
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
    task_monitor: Arc<std::sync::Mutex<TaskMonitor>>,
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
        tokio::spawn(async move {
            loop {
                let event_recv = task_monitor_event_receiver.recv().await;
                match event_recv {
                    Some(event) => {
                        // log_debug!(logger, "TaskMonitor event: {:?}", event);
                        // Handle the event as needed
                    }
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
            logger: Logger::new_for_instance(name.clone()),
            engine: engine.clone(),
            topic: format!("{}/{}", engine.root_topic(), name),
            settings: settings,
            actions: Arc::new(Mutex::new(actions)),
            state: Arc::new(Mutex::new(State::Booting)),
            state_change_notifier: Arc::new(Notify::new()),
            notification_channel: notification_channel,
            reset_signal: Arc::new(Notify::new()),
            task_monitor: Arc::new(std::sync::Mutex::new(task_monitor)),
        }
    }

    ///
    ///
    pub fn task_monitor_sender(&self) -> Sender<NamedTaskHandle> {
        self.task_monitor.lock().unwrap().handle_sender()
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

        // println!("new state !!! {:?}", new_state.clone());

        // Alert monitoring device "_"
        // if let Some(r_notifier) = &mut self.r_notifier {
        //     r_notifier
        //         .try_send(StateNotification::new(self.topic.clone(), new_state.clone()).into())
        //         .unwrap();
        // }
        // else {
        //     self.logger
        //         .debug("!!!!!!! DEBUG !!!!!!! r_notifier is 'None'");
        // }

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
        AttributeServerBuilder::new(self.engine.clone(), None, self.notification_channel.clone())
            .with_topic(format!("{}/{}", self.topic, name.into()))
    }

    /// Override
    ///
    fn monitor_task(&self, named_task_handle: NamedTaskHandle) {
        self.task_monitor_sender()
            .try_send(named_task_handle)
            .unwrap();
    }
}
