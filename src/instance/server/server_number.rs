use std::{future::Future, sync::Arc};
use tokio::sync::Mutex;

use super::server::AttServer;
use crate::{generic_att_server_methods, AttributeServerBuilder, Error, Logger, NumberCodec};

///
///
///
#[derive(Clone)]
pub struct NumberAttServer {
    /// Local logger
    ///
    logger: Logger,

    ///
    /// Inner server implementation
    pub inner: Arc<Mutex<AttServer<NumberCodec>>>,
}

impl NumberAttServer {
    //
    // Require inner member
    generic_att_server_methods!();

    ///
    ///
    pub fn r#type() -> String {
        "number".to_string()
    }

    ///
    ///
    ///
    pub fn new(builder: AttributeServerBuilder) -> Self {
        let obj = AttServer::<NumberCodec>::from(builder);
        Self {
            logger: obj.logger.clone(),
            inner: Arc::new(Mutex::new(obj)),
        }
    }

    ///
    /// Get the value of the attribute
    /// If None, the first value is not yet received
    ///
    pub async fn pop_cmd_as_i64(&mut self) -> Option<i64> {
        self.inner
            .lock()
            .await
            .pop_cmd()
            .and_then(|v| v.value.as_i64())
    }

    /// Set the value of the attribute
    ///
    pub async fn set_from_i64(&self, value: i64) -> Result<(), Error> {
        self.inner.lock().await.set(value.into()).await?;
        Ok(())
    }
}
