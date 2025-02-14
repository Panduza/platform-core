pub mod settings;
use async_trait::async_trait;
use bytes::Bytes;
use futures::FutureExt;
use panduza::pubsub::PubSubOperator;
use panduza::router::RouterHandler;
pub use settings::ReactorSettings;

use crate::{AttributeBuilder, Error, MessageDispatcher, MessageHandler, TaskResult, TaskSender};
use crate::{MessageClient, Notification};
use chrono::prelude::*;
use rand::distributions::Alphanumeric;
use rand::Rng;
use rumqttc::AsyncClient;
use rumqttc::{MqttOptions, QoS};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

struct PzaScanMessageHandler {
    message_client: MessageClient,
}

// #[async_trait]
// impl MessageHandler for PzaScanMessageHandler {
//     async fn on_message(&mut self, _incomming_data: &Bytes) -> Result<(), Error> {
//         // let hostname = hostname::get().unwrap().to_string_lossy().to_string();
//         let now = Utc::now();

//         self.message_client
//             .publish(
//                 format!("pza"),
//                 QoS::AtLeastOnce,
//                 false,
//                 format!("{}", now.timestamp_millis()),
//             )
//             .await
//             .map_err(|e| Error::PublishError {
//                 topic: "pza".to_string(),
//                 pyl_size: now.timestamp_millis().to_string().len(),
//                 cause: e.to_string(),
//             })?;
//         Ok(())
//     }
// }

/// The reactor is the main structure that will handle the connections and the events
///
/// All the attribute and objects will be powered by the reactor
///
#[derive(Clone)]
pub struct Reactor<O: PubSubOperator> {
    is_started: bool,

    /// Root topic (namespace/pza)
    root_topic: String,

    scan_handler: Option<Arc<Mutex<PzaScanMessageHandler>>>,

    ///
    ///
    router: RouterHandler<O>,
}

impl<O: PubSubOperator> Reactor<O> {
    /// Create a new Reactor
    ///
    /// # Arguments
    ///
    /// * `core` - The core of the reactor
    ///
    pub fn new(_settings: ReactorSettings) -> Self {
        // let data = ;

        // Server hostname
        // let hostname = hostname::get().unwrap().to_string_lossy().to_string();

        Self {
            is_started: false,
            root_topic: format!("pza"),
            message_client: None,
            message_dispatcher: Arc::new(Mutex::new(MessageDispatcher::new())),
            scan_handler: None,
        }
    }

    pub fn root_topic(&self) -> String {
        self.root_topic.clone()
    }

    fn generate_random_string(length: usize) -> String {
        let rng = rand::thread_rng();
        rng.sample_iter(Alphanumeric)
            .take(length)
            .map(|c| c as char)
            .collect()
    }

    pub fn start(
        &mut self,
        mut main_task_sender: TaskSender<TaskResult>,
    ) -> Result<(), crate::Error> {
        if self.is_started {
            return Ok(());
        }

        let mut mqttoptions = MqttOptions::new(
            format!("rumqtt-sync-{}", Self::generate_random_string(5)),
            "localhost",
            1883,
        );
        mqttoptions.set_keep_alive(Duration::from_secs(3));

        let (client, event_loop) = AsyncClient::new(mqttoptions, 100);

        self.message_client = Some(client.clone());

        self.scan_handler = Some(Arc::new(Mutex::new(PzaScanMessageHandler {
            message_client: client.clone(),
        })));

        let h = self.scan_handler.as_ref().unwrap().clone();
        let dispatcher = self.message_dispatcher.clone();
        let mut message_engine = MessageEngine::new(self.message_dispatcher.clone(), event_loop);
        main_task_sender.spawn_with_name(
            "REACTOR CORE",
            async move {
                dispatcher
                    .lock()
                    .await
                    .register_message_attribute("pza".to_string(), h);
                client.subscribe("pza", QoS::AtLeastOnce).await.unwrap();
                message_engine.run().await;
                println!("!!!!!!!!!!!! ReactorCore STOP not runiing !!!!!!!!!!!!!!!!!!!!!!");
                Ok(())
            }
            .boxed(),
        )?;

        self.is_started = true;
        Ok(())
    }

    pub fn create_new_attribute(
        &self,
        // device_dyn_info: Option<ThreadSafeInfoDynamicDeviceStatus>,
        r_notifier: Option<Sender<Notification>>,
    ) -> AttributeBuilder {
        AttributeBuilder::new(
            None,
            self.message_client.as_ref().unwrap().clone(),
            Arc::downgrade(&self.message_dispatcher),
            r_notifier,
        )
    }
}

/// Start the reactor
///
pub async fn new_reactor(options: ReactorOptions) -> Result<Reactor<impl PubSubOperator>, String> {
    let router = new_mqtt_router(options.pubsub_options).map_err(|e| e.to_string())?;

    let handler = router.start(None).await.unwrap();

    let structure_data_receiver = handler.register_listener("pza/_/structure/att", 5).await?;

    let structure = Structure::new(structure_data_receiver);
    let structure_initialized = structure.initialized_notifier();

    structure_initialized.notified().await;

    Ok(Reactor::new(structure, handler))
}
