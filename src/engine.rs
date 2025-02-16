pub mod options;
use options::EngineOptions;

use panduza::pubsub::Operator;
use panduza::router::RouterHandler;

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

/// The engine is the core object that will handle the connections and the events
///
/// All the attribute and objects will be powered by the engine
///
#[derive(Clone)]
pub struct Engine {
    /// Engine works on router objects
    ///
    router: RouterHandler,
}

impl Engine {
    /// Create a new Reactor
    ///
    /// # Arguments
    ///
    /// * `core` - The core of the reactor
    ///
    pub fn new(router: RouterHandler) -> Self {
        // let data = ;

        // Server hostname
        // let hostname = hostname::get().unwrap().to_string_lossy().to_string();

        Self { router: router }
    }

    // pub fn root_topic(&self) -> String {
    //     self.root_topic.clone()
    // }

    // pub fn start(
    //     &mut self,
    //     mut main_task_sender: TaskSender<TaskResult>,
    // ) -> Result<(), crate::Error> {
    //     if self.is_started {
    //         return Ok(());
    //     }

    //     let mut mqttoptions = MqttOptions::new(
    //         format!("rumqtt-sync-{}", Self::generate_random_string(5)),
    //         "localhost",
    //         1883,
    //     );
    //     mqttoptions.set_keep_alive(Duration::from_secs(3));

    //     let (client, event_loop) = AsyncClient::new(mqttoptions, 100);

    //     self.message_client = Some(client.clone());

    //     self.scan_handler = Some(Arc::new(Mutex::new(PzaScanMessageHandler {
    //         message_client: client.clone(),
    //     })));

    //     let h = self.scan_handler.as_ref().unwrap().clone();
    //     let dispatcher = self.message_dispatcher.clone();
    //     let mut message_engine = MessageEngine::new(self.message_dispatcher.clone(), event_loop);
    //     main_task_sender.spawn_with_name(
    //         "REACTOR CORE",
    //         async move {
    //             dispatcher
    //                 .lock()
    //                 .await
    //                 .register_message_attribute("pza".to_string(), h);
    //             client.subscribe("pza", QoS::AtLeastOnce).await.unwrap();
    //             message_engine.run().await;
    //             println!("!!!!!!!!!!!! ReactorCore STOP not runiing !!!!!!!!!!!!!!!!!!!!!!");
    //             Ok(())
    //         }
    //         .boxed(),
    //     )?;

    //     self.is_started = true;
    //     Ok(())
    // }

    // pub fn create_new_attribute(
    //     &self,
    //     // device_dyn_info: Option<ThreadSafeInfoDynamicDeviceStatus>,
    //     r_notifier: Option<Sender<Notification>>,
    // ) -> AttributeBuilder {
    //     AttributeBuilder::new(
    //         None,
    //         self.message_client.as_ref().unwrap().clone(),
    //         Arc::downgrade(&self.message_dispatcher),
    //         r_notifier,
    //     )
    // }
}

/// Create and Start the engine
///
pub async fn new_engine(options: EngineOptions) -> Result<Engine, String> {
    //
    // Create MQTT router
    let router =
        panduza::router::new_mqtt_router(options.pubsub_options).map_err(|e| e.to_string())?;

    //
    // Start the router and keep the operation handler
    let router_handler = router.start(None).await.unwrap();

    //
    // Finalize the engine
    Ok(Engine::new(router_handler))
}
