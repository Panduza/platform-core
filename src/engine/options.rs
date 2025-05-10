use panduza::pubsub;

#[derive(Debug)]
/// Options of the platform engine
///
pub struct EngineOptions {
    /// Options for the pub sub connection
    ///
    pub pubsub_options: pubsub::Options,
}

impl EngineOptions {
    pub fn new<T: Into<String>>(ip: T, port: u16) -> Self {
        Self {
            pubsub_options: pubsub::Options {
                ip: ip.into(),
                port: port,
            },
        }
    }
}
