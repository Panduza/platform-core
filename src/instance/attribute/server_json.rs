use std::{future::Future, sync::Arc};
use tokio::sync::Mutex;

use super::server::AttServer;
use crate::{
    generic_att_server_methods, instance::element::Element, AttributeServerBuilder, Error,
    JsonCodec, Logger,
};

///
///
///
#[derive(Clone)]
pub struct JsonAttServer {
    /// Local logger
    ///
    logger: Logger,

    /// Inner server implementation
    ///
    pub inner: Arc<Mutex<AttServer<JsonCodec>>>,
}

impl JsonAttServer {
    //
    // Require inner member
    generic_att_server_methods!();

    /// Clone as an element object
    ///
    pub fn clone_as_element(&self) -> Element {
        Element::AsJson(self.clone())
    }

    ///
    ///
    pub fn r#type() -> String {
        "json".to_string()
    }

    ///
    ///
    ///
    pub fn new(builder: AttributeServerBuilder) -> Self {
        let obj = AttServer::<JsonCodec>::from(builder);
        Self {
            logger: obj.logger.clone(),
            inner: Arc::new(Mutex::new(obj)),
        }
    }

    ///
    /// Get the value of the attribute
    /// If None, the first value is not yet received
    ///
    pub async fn pop_cmd(&mut self) -> Option<serde_json::Value> {
        self.inner
            .lock()
            .await
            .pop_cmd()
            .and_then(|v| Some(v.value))
    }

    /// Set the value of the attribute
    ///
    pub async fn set(&self, value: serde_json::Value) -> Result<(), Error> {
        self.inner
            .lock()
            .await
            .set(JsonCodec { value: value })
            .await?;
        Ok(())
    }
}
