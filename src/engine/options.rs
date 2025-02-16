use panduza::pubsub;

#[derive(Default, Debug)]
/// Options of the platform engine
///
pub struct EngineOptions {
    /// Options for the pub sub connection
    ///
    pub pubsub_options: pubsub::Options,
}
