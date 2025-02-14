use crate::{Error, Instance, ProductionOrder, Props};
use async_trait::async_trait;
use bytes::Bytes;
use std::fmt::Debug;

/// Actions that are specific for each driver
///
#[async_trait]
pub trait DriverOperations: Send + Sync {
    ///
    /// Mount driver instance and give him its structure
    ///
    async fn mount(&mut self, mut instance: Instance) -> Result<(), Error>;

    ///
    /// This instance crashed, got an error or is not available anymore
    /// This function must monitor reboot condition and await them
    /// Once this function return, the instance will reboot
    ///
    async fn wait_reboot_event(&mut self, mut instance: Instance);
}

///
///
///
pub trait Scanner: Send {
    ///
    ///
    ///
    fn name(&self) -> String;
    ///
    ///
    ///
    fn scan(&self) -> Vec<ProductionOrder>;
}

/// Trait to manage an message attribute (MQTT)
/// Sync version
#[async_trait]
pub trait MessageHandler: Send + Sync {
    ///
    /// Triggered on each incoming message
    ///
    async fn on_message(&mut self, data: &Bytes) -> Result<(), Error>;
}

/// Encoder Decoder for message payload
///
pub trait MessageCodec: PartialEq + Debug + Sync + Send + Clone + 'static {
    ///
    /// Decode data
    ///
    fn from_message_payload(data: &Bytes) -> Result<Self, Error>;
    ///
    /// Encode data
    ///
    fn into_message_payload(&self) -> Result<Vec<u8>, Error>;

    fn typee() -> String;
}
