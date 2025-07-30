pub mod options;
use options::EngineOptions;

use panduza::{
    pubsub::{self, new_connection}, // router::{DataReceiver, Router, RouterHandler},
};
use zenoh::pubsub::Publisher;
use zenoh::pubsub::Subscriber;
use zenoh::sample::Sample;
use zenoh::{handlers::FifoChannelHandler, Session};

/// The engine is the core object that will handle the connections and the events
///
/// All the attribute and objects will be powered by the engine
///
#[derive(Clone)]
pub struct Engine {
    /// Engine works on router objects
    ///
    pub session: Session,

    /// Namespace of the engine
    ///
    pub namespace: Option<String>,
}

impl Engine {
    /// Create a new Reactor
    ///
    /// # Arguments
    ///
    /// * `core` - The core of the reactor
    ///
    pub fn new(session: Session, namespace: Option<String>) -> Self {
        // let data = ;

        // Server hostname
        // let hostname = hostname::get().unwrap().to_string_lossy().to_string();

        Self {
            session: session,
            namespace: namespace,
        }
    }

    ///
    ///
    pub fn root_topic(&self, namespace: Option<String>) -> String {
        println!("namespace: {:?}", namespace);
        format!(
            "{}pza",
            namespace.map_or("".to_string(), |ns| if ns.is_empty() {
                "".to_string()
            } else {
                format!("{}/", ns)
            })
        )
    }

    /// Register
    ///
    pub async fn register_listener<A: Into<String> + 'static>(
        &self,
        topic: A,
        _channel_size: usize,
    ) -> Subscriber<FifoChannelHandler<Sample>> {
        let topic_str: String = topic.into();
        // let topic_prefixless = topic_str.strip_prefix("Zenoh/").unwrap_or(&topic_str);

        // println!("topic_prefixless: {}", topic_prefixless);
        println!("topic: {}", topic_str.clone());

        self.session.declare_subscriber(topic_str).await.unwrap()
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

    let session = new_connection(options.pubsub_options.clone())
        .await
        .map_err(|e| e.to_string())?;

    //
    // Start the router and keep the operation handler
    // let router_handler = router.start(None).unwrap();

    //
    // Finalize the engine
    Ok(Engine::new(session, options.pubsub_options.namespace))
}

/// The goal of this object is to provide a tmp object that
/// does not use tokio:spawn, to be able to prepare the context.
/// Before starting a tokio context.
///
pub struct EngineBuilder {
    options: EngineOptions,
}

impl EngineBuilder {
    /// Create and Start the engine
    ///
    /// This function MUST absolutely not be async !
    /// It will be used in plugin sync context
    ///
    pub fn new(options: EngineOptions) -> Self {
        Self {
            // options: options,
            options: options,
        }
    }

    pub async fn build(self) -> Engine {
        let namespace = self.options.pubsub_options.namespace.clone();
        let session = new_connection(self.options.pubsub_options).await.unwrap();
        //
        // Finalize the engine
        Engine::new(session, namespace)
    }
}
