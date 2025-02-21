use crate::{Actions, Error, Props};

/// Trait to define an instance producer
/// Its job is to produce an instance of a driver
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
    fn produce(&self) -> Result<Box<dyn Actions>, Error>;
}
