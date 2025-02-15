use crate::{Error, Instance, ProductionOrder, Props};
use async_trait::async_trait;
use bytes::Bytes;
use std::fmt::Debug;

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
