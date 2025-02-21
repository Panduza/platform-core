use std::{future::Future, sync::Arc};
use tokio::sync::Mutex;

use super::server::AttServer;
use crate::{
    generic_att_server_methods, AttributeServerBuilder, Error, Logger, MemoryCommandCodec,
};

///
///
///
#[derive(Clone)]
pub struct MemoryCommandAttServer {
    /// Local logger
    ///
    logger: Logger,

    ///
    /// Inner server implementation
    pub inner: Arc<Mutex<AttServer<MemoryCommandCodec>>>,
}

impl MemoryCommandAttServer {
    //
    // Require inner member
    generic_att_server_methods!();

    ///
    ///
    pub fn r#type() -> String {
        "memory_command".to_string()
    }

    ///
    ///
    ///
    pub fn new(builder: AttributeServerBuilder) -> Self {
        let obj = AttServer::<MemoryCommandCodec>::from(builder);
        Self {
            logger: obj.logger.clone(),
            inner: Arc::new(Mutex::new(obj)),
        }
    }

    ///
    /// Get the value of the attribute
    /// If None, the first value is not yet received
    ///
    pub async fn pop_cmd(&mut self) -> Option<MemoryCommandCodec> {
        self.inner.lock().await.pop_cmd()
    }

    /// Set the value of the attribute
    ///
    pub async fn set(&self, value: MemoryCommandCodec) -> Result<(), Error> {
        self.inner.lock().await.set(value).await?;
        Ok(())
    }
}
