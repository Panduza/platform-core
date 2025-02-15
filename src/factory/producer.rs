use crate::{Actions, Error, Props};
use panduza::pubsub::PubSubOperator;

/// Trait to define an instance producer
/// Its job is to produce an instance of a driver
///
pub trait Producer<O: PubSubOperator>: Send {
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
    fn produce(&self) -> Result<Box<dyn Actions<O>>, Error>;
}
