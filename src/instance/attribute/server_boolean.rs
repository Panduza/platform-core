use super::server::AttServer;

use crate::{
    generic_att_server_methods, instance::element::Element, AttributeBuilder, BooleanCodec, Error,
    Logger,
};

use std::{future::Future, sync::Arc};
use tokio::sync::Mutex;

///
///
///
#[derive(Clone)]
pub struct BooleanAttServer {
    /// Local logger
    ///
    logger: Logger,

    /// Inner server implementation
    ///
    pub inner: Arc<Mutex<AttServer<BooleanCodec>>>,
}

impl BooleanAttServer {
    //
    // Require inner member
    generic_att_server_methods!();

    /// Clone as an element object
    ///
    pub fn clone_as_element(&self) -> Element {
        Element::AsBoolean(self.clone())
    }

    ///
    ///
    pub fn r#type() -> String {
        "boolean".to_string()
    }

    ///
    ///
    pub fn new(builder: AttributeBuilder) -> Self {
        let obj = AttServer::<BooleanCodec>::from(builder);
        Self {
            logger: obj.logger.clone(),
            inner: Arc::new(Mutex::new(obj)),
        }
    }

    ///
    /// Get the value of the attribute
    /// If None, the first value is not yet received
    ///
    pub async fn pop_cmd(&mut self) -> Option<bool> {
        self.inner
            .lock()
            .await
            .pop_cmd()
            .and_then(|v| Some(v.value))
    }

    ///
    /// Get the value of the attribute
    /// If None, the first value is not yet received
    ///
    pub async fn get_last_cmd(&self) -> Option<bool> {
        return self
            .inner
            .lock()
            .await
            .get_last_cmd()
            .and_then(|v| Some(v.value));
    }

    /// Set the value of the attribute
    ///
    pub async fn set(&self, value: bool) -> Result<(), Error> {
        self.inner
            .lock()
            .await
            .set(BooleanCodec { value: value })
            .await?;
        Ok(())
    }
}
