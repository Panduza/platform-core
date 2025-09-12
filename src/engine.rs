use zenoh::handlers::FifoChannelHandler;
use zenoh::pubsub::{Publisher, Subscriber};
use zenoh::sample::Sample;
use zenoh::Session;

// Temporary stub function until panduza::connection is fully implemented
async fn new_connection(_config: panduza::Config) -> Result<Session, zenoh::Error> {
    // For now, create a basic zenoh session
    // TODO: Replace with proper panduza::connection::create_client_connection implementation
    // TODO: Use the provided config to configure the session (ip, port, certificates, etc.)
    zenoh::open(zenoh::Config::default()).await
}

/// Error type for engine operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Zenoh session error: {0}")]
    Session(#[from] zenoh::Error),
    #[error("Publisher registration failed: {0}")]
    Publisher(String),
    #[error("Listener registration failed: {0}")]
    Listener(String),
}

/// The Engine struct is the main entity that manages the platform's communication infrastructure.
///
/// The Engine module is the core component of the Panduza platform that handles connections,
/// events, and communication through the Zenoh protocol. It serves as the foundational layer
/// that powers all attributes and objects within the system.
///
/// Characteristics:
/// - Implements Clone trait for easy duplication across different contexts
/// - Serves as the central hub for all pub/sub operations
#[derive(Clone)]
pub struct Engine {
    /// A Zenoh session that handles the underlying communication protocol
    pub session: Session,

    /// An optional namespace that provides logical separation for different engine instances
    pub namespace: Option<String>,
}

impl Engine {
    /// Creates a new Engine instance with the provided Zenoh session and optional namespace
    /// Initializes the core communication infrastructure
    ///
    /// # Arguments
    ///
    /// * `session` - A Zenoh session that handles the underlying communication protocol
    /// * `namespace` - An optional namespace that provides logical separation for different engine instances
    ///
    pub fn new(session: Session, namespace: Option<String>) -> Self {
        Self { session, namespace }
    }

    /// Generates the root topic path for the engine
    /// Combines the provided namespace (or engine's namespace) with the "pza" suffix
    /// Returns a properly formatted topic string for Zenoh communication
    /// Handles empty namespaces gracefully by omitting the namespace prefix
    pub fn root_topic(&self, namespace: Option<String>) -> String {
        let ns = namespace.or_else(|| self.namespace.clone());
        match ns {
            Some(namespace) if !namespace.is_empty() => format!("{}/pza", namespace),
            _ => "pza".to_string(),
        }
    }

    /// Registers a subscriber for listening to messages on a specific topic
    /// Returns a Zenoh subscriber with FIFO channel handling
    /// Provides asynchronous message reception capabilities
    pub async fn register_listener<A: Into<String> + 'static>(
        &self,
        topic: A,
        _channel_size: usize,
    ) -> Subscriber<FifoChannelHandler<Sample>> {
        self.session.declare_subscriber(topic.into()).await.unwrap()
    }

    /// Registers a publisher for sending messages to a specific topic
    /// Returns a Zenoh publisher wrapped in a Result for error handling
    /// Enables asynchronous message publishing capabilities
    pub async fn register_publisher<A: Into<String> + 'static>(
        &self,
        topic: A,
    ) -> Result<Publisher, Error> {
        self.session
            .declare_publisher(topic.into())
            .await
            .map_err(|e| Error::Publisher(format!("Failed to register publisher: {}", e)))
    }
}

/// The EngineBuilder provides a non-async way to prepare Engine configuration
/// before entering a Tokio runtime context.
///
/// Purpose:
/// - Allows synchronous preparation of engine configuration
/// - Enables use in plugin contexts where async operations are not immediately available
/// - Defers the actual async connection establishment until explicitly requested
pub struct EngineBuilder {
    config: panduza::Config,
}

impl EngineBuilder {
    /// Creates a new builder instance with the specified config
    /// Synchronous operation suitable for plugin initialization
    pub fn new(config: panduza::Config) -> Self {
        Self { config }
    }

    /// Consumes the builder and creates the actual Engine instance
    /// Establishes the Zenoh connection and finalizes the engine setup
    /// Must be called within an async context
    pub async fn build(self) -> Result<Engine, Error> {
        // Create Zenoh session using the config
        let session = new_connection(self.config.clone())
            .await
            .map_err(Error::Session)?;

        // Extract namespace from config
        let namespace = None; // TODO: Extract namespace from panduza::Config when available

        Ok(Engine::new(session, namespace))
    }
}
