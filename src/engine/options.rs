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
        root_ca_certificate: T,
        connect_certificate: T,
        connect_private_key: T,
        namespace: Option<T>,
    ) -> Self {
        println!("namespace dans new engine options: {:?}", namespace);
        Self {
            pubsub_options: pubsub::Options {
                ip: ip.into(),
                port: port,
                root_ca_certificate: root_ca_certificate.into(),
                connect_certificate: connect_certificate.into(),
                connect_private_key: connect_private_key.into(),
                namespace: namespace.map(|n| n.into()),
            },
        }
    }
}
