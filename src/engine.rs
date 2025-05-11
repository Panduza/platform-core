pub mod options;
use options::EngineOptions;

use panduza::{
    pubsub::{self, new_connection}, // router::{DataReceiver, Router, RouterHandler},
};
use zenoh::pubsub::Publisher;
use zenoh::pubsub::Subscriber;
use zenoh::sample::Sample;
use zenoh::{handlers::FifoChannelHandler, Session};

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
    pub session: Session,
}

impl Engine {
    /// Create a new Reactor
    ///
    /// # Arguments
    ///
    /// * `core` - The core of the reactor
    ///
    pub fn new(session: Session) -> Self {
        // let data = ;

        // Server hostname
        // let hostname = hostname::get().unwrap().to_string_lossy().to_string();

        Self { session: session }
    }

    ///
    ///
    pub fn root_topic(&self) -> String {
        "pza".to_string()
        // self.root_topic.clone()
    }

    /// Register
    ///
    pub async fn register_listener<A: Into<String> + 'static>(
        &self,
        topic: A,
        channel_size: usize,
    ) -> Subscriber<FifoChannelHandler<Sample>> {
        self.session.declare_subscriber(topic.into()).await.unwrap()
    }

    ///
    ///
    pub async fn register_publisher<A: Into<String> + 'static>(
        &self,
        topic: A,
    ) -> Result<Publisher, pubsub::Error> {
        Ok(self.session.declare_publisher(topic.into()).await.unwrap())
    }

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
}

/// Create and Start the engine
///
pub async fn new_engine(options: EngineOptions) -> Result<Engine, String> {
    //
    // Create MQTT router
    // let router = panduza::router::new_router(options.pubsub_options).map_err(|e| e.to_string())?;

    let session = new_connection(options.pubsub_options)
        .await
        .map_err(|e| e.to_string())?;

    //
    // Start the router and keep the operation handler
    // let router_handler = router.start(None).unwrap();

    //
    // Finalize the engine
    Ok(Engine::new(session))
}

// / The goal of this object is to provide a tmp object that
// / does not use tokio:spawn, to be able to prepare the context.
// / Before starting a tokio context.
// /

pub struct EngineBuilder {
    // options: EngineOptions,
    session: Session,
}

impl EngineBuilder {
    /// Create and Start the engine
    ///
    pub async fn new(options: EngineOptions) -> Self {
        //
        // Create router
        let session = new_connection(options.pubsub_options).await.unwrap();

        Self {
            // options: options,
            session: session,
        }
    }

    pub fn build(self) -> Engine {
        //
        // Finalize the engine
        Engine::new(self.session)
    }
}
