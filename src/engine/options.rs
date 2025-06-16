use panduza::pubsub;
use std::fmt::Debug;

#[derive(Debug)]
/// Options of the platform engine
///
pub struct EngineOptions {
    /// Options for the pub sub connection
    ///
    pub pubsub_options: pubsub::Options,
}

impl EngineOptions {
    pub fn new<T: Into<String> + Debug>(
        ip: T,
        port: u16,
        ca_certificate: T,
        namespace: Option<T>,
    ) -> Self {
        println!("namespace dans new engine options: {:?}", namespace);
        Self {
            pubsub_options: pubsub::Options {
                ip: ip.into(),
                port: port,
                ca_certificate: ca_certificate.into(),
                namespace: namespace.map(|n| n.into()),
            },
        }
    }
}
