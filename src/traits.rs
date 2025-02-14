use crate::{Instance, Error, ProductionOrder, Props};
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

/// Trait to define a driver producer
/// Its job is to produce instanciation of drivers
///
pub trait Producer: Send {
    /// Driver Manufacturer
    ///
    fn manufacturer(&self) -> String;

    /// Driver Model
    ///
    fn model(&self) -> String;

    /// Driver Description
    ///
    /// What the driver do ?
    ///
    fn description(&self) -> String;

    /// Device settings properties
    ///
    fn props(&self) -> Props;

    /// Produce a new instance of the device actions
    ///
    fn produce(&self) -> Result<Box<dyn DriverOperations>, Error>;
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
