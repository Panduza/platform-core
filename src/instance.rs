pub mod actions;
pub mod attribute_builder;
pub mod class;
pub mod class_builder;
pub mod container;
pub mod server;

use async_trait::async_trait;
use attribute_builder::AttributeServerBuilder;
use class_builder::ClassBuilder;
use panduza::task_monitor::{NamedTaskHandle, TaskHandle};
use panduza::{InstanceState, TaskMonitor};


use crate::engine::Engine;
use crate::log_debug;
use crate::log_error;
use crate::log_trace;
use crate::Actions;
use crate::InstanceSettings;
use crate::Logger;
use crate::Notification;
use crate::StateNotification;
pub use container::Container;
use crate::log_info;
// use class_builder::ClassBuilder;

use serde::{Deserialize, Serialize};
use std::{fmt::Display, sync::Arc};
use tokio::sync::{mpsc::Sender, Notify, Mutex};

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
    state: Arc<Mutex<InstanceState>>,

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
        // Create a task monitor for the instance
        let (task_monitor, task_monitor_event_receiver) =
            TaskMonitor::new(format!("INSTANCE/{}", name));

        //
        // Create instance
        let instance = Instance {
            logger: Logger::new_for_instance(name.clone()),
            engine: engine.clone(),
            topic: format!("{}/{}", engine.root_topic(), name),
            settings,
            actions: Arc::new(Mutex::new(actions)),
            state: Arc::new(Mutex::new(InstanceState::Booting)),
            state_change_notifier: Arc::new(Notify::new()),
            notification_channel: notification_channel.clone(),
            reset_signal: Arc::new(Notify::new()),
            task_monitor: task_monitor,
        };

        //
        // Spawn task monitor handler with captures of what's needed
        tokio::spawn(handle_task_monitor_events(
            task_monitor_event_receiver,
            instance.logger.clone(),
            instance.state.clone(),
            instance.state_change_notifier.clone(),
            notification_channel.clone(),
            instance.topic.clone(),
        ));

        instance
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
        self.move_to_state(InstanceState::Booting).await;

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
                InstanceState::Booting => {
                    // if let Some(mut info_pack) = self.info_pack.clone() {
                    //     self.logger.debug("FSM try to add_deivce in info pack");
                    //     self.info_dyn_dev_status = Some(info_pack.add_device(self.name()).await);
                    //     self.logger.debug("FSM finish info pack");
                    // } else {
                    //     self.logger.debug("FSM NO INFO PACK !");
                    // }
                    self.move_to_state(InstanceState::Initializating).await;
                }
                InstanceState::Connecting => {} // wait for reactor signal
                InstanceState::Initializating => {
                    //
                    // Try to mount the device
                    let mount_result = self.actions.lock().await.mount(self.clone()).await;
                    //
                    // Manage mount result
                    match mount_result {
                        Ok(_) => {
                            self.logger.debug("FSM Mount Success ");
                            self.move_to_state(InstanceState::Running).await;
                        }
                        Err(e) => {
                            log_error!(self.logger, "Instance Mount Failure '{:?}'", e);
                            self.move_to_state(InstanceState::Error).await;
                        }
                    }
                }
                InstanceState::Running => {} // do nothing, watch for inner tasks
                InstanceState::Error => {
                    self.task_monitor.cancel_all_monitored_tasks().await;
                    //
                    // Wait before reboot
                    self.actions
                        .lock()
                        .await
                        .wait_reboot_event(self.clone())
                        .await;
                    self.logger.info("try to reboot");
                    self.move_to_state(InstanceState::Initializating).await;
                }
                InstanceState::Warning => {}
                InstanceState::Cleaning => {}
                InstanceState::Stopping => {}
                InstanceState::Undefined => {}
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
        log_info!(self.logger(), "GO ERROR");
        self.move_to_state(InstanceState::Error).await;
    }

    ///
    /// Function to change the current state of the device FSM
    ///
    pub async fn move_to_state(&mut self, new_state: InstanceState) {
        // Set the new state
        *self.state.lock().await = new_state.clone();

        // Alert monitoring device "_"
        if let Err(err) = self
            .notification_channel
            .send(StateNotification::new(self.topic.clone(), new_state.clone()).into())
            .await
        {
            log_error!(self.logger, "Failed to send state notification: {}", err);
        }

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

/// Handle task monitor events
///
async fn handle_task_monitor_events(
    mut event_receiver: tokio::sync::mpsc::Receiver<panduza::task_monitor::Event>,
    logger: Logger,
    state: Arc<Mutex<InstanceState>>,
    state_change_notifier: Arc<Notify>,
    notification_channel: Sender<Notification>,
    topic: String,
) {
    loop {
        let event_recv = event_receiver.recv().await;

        match event_recv {
            Some(event) => {
                match &event {
                    //
                    // An error occurred in the task monitor
                    panduza::task_monitor::Event::TaskMonitorError(err_msg) => {
                        log_error!(logger, "Task monitor error: {}", err_msg);
                    }

                    // Regrouper les traitements d'erreurs similaires
                    panduza::task_monitor::Event::TaskStopWithPain(event_body)
                    | panduza::task_monitor::Event::TaskPanicOMG(event_body) => {
                        // Déterminer le type d'erreur pour le logging
                        let error_type = match event {
                            panduza::task_monitor::Event::TaskStopWithPain(_) => "Error",
                            _ => "Panic",
                        };

                        // Logger l'événement
                        log_error!(
                            logger,
                            "{} on task {} - {}",
                            error_type,
                            event_body.task_name,
                            event_body
                                .error_message
                                .clone()
                                .unwrap_or_else(|| "No error details".into())
                        );

                        // Mettre à jour l'état
                        *state.lock().await = InstanceState::Error;

                        // Envoyer la notification
                        if let Err(err) = notification_channel
                            .send(
                                StateNotification::new(topic.clone(), InstanceState::Error).into(),
                            )
                            .await
                        {
                            log_error!(logger, "Failed to send notification: {}", err);
                        }

                        // Notifier la machine à états
                        state_change_notifier.notify_one();
                    }

                    // Gérer les autres types d'événements
                    panduza::task_monitor::Event::TaskCreated(event_body) => {
                        log_trace!(logger, "Task created: {}", event_body.task_name);
                    }

                    panduza::task_monitor::Event::TaskStopProperly(event_body) => {
                        log_trace!(
                            logger,
                            "Task completed successfully: {}",
                            event_body.task_name
                        );
                    }

                    panduza::task_monitor::Event::NoMoreTask => {
                        log_trace!(logger, "No more tasks to monitor");
                    }
                }
            }
            None => {
                log_debug!(logger, "TaskMonitor channel closed, stopping monitor task");
                break;
            }
        }
    }
}
