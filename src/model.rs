use crate::Error;
use async_trait::async_trait;

#[async_trait]
///
///
pub trait BooleanAccessorModel: Sync + Send {
    ///
    ///
    async fn get_boolean_at(&mut self, index: usize) -> Result<bool, Error>;
    ///
    ///
    async fn set_boolean_at(&mut self, index: usize, value: bool) -> Result<(), Error>;
}
