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

#[async_trait]
///
///
pub trait VectorF32AccessorModel: Sync + Send {
    ///
    ///
    async fn get_vectorf32_at(&mut self, index: usize) -> Result<Vec<f32>, Error>;
    ///
    ///
    async fn set_vectorf32_at(&mut self, index: usize, value: Vec<f32>) -> Result<(), Error>;
}

// acquisitor -> data ro + trigger
// one or multiple vector of f32
