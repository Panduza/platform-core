use super::server::AttServer;
use crate::{generic_att_server_methods, AttributeBuilder, Error, Logger, StringCodec};

use std::{future::Future, sync::Arc};
use tokio::sync::Mutex;

///
///
///
#[derive(Clone)]
pub struct EnumAttServer {
    /// Local logger
    ///
    logger: Logger,

    ///
    /// Inner server implementation
    pub inner: Arc<Mutex<AttServer<StringCodec>>>,

    ///
    ///
    ///
    choices: Vec<String>,
}

impl EnumAttServer {
    //
    // Require inner member
    generic_att_server_methods!();

    ///
    ///
    ///
    pub fn r#type() -> String {
        "enum".to_string()
    }

    ///
    ///
    ///
    pub fn new(builder: AttributeBuilder, choices: Vec<String>) -> Self {
        let obj = AttServer::<StringCodec>::from(builder);
        Self {
            logger: obj.logger.clone(),
            inner: Arc::new(Mutex::new(obj)),
            choices: choices,
        }
    }

    ///
    /// Get the value of the attribute
    /// If None, the first value is not yet received
    ///
    pub async fn pop_cmd(&mut self) -> Option<Result<String, Error>> {
        let v_brute = self.inner.lock().await.pop_cmd();
        match v_brute {
            Some(v) => {
                if self.choices.contains(&v.value) {
                    Some(Ok(v.value))
                } else {
                    Some(Err(Error::EnumOutOfChoices(format!(
                        "{:?} is not in {:?}",
                        v.value, self.choices
                    ))))
                }
            }
            None => None,
        }
    }

    /// Set the value of the attribute
    ///
    pub async fn set<S: Into<String>>(&self, value: S) -> Result<(), Error> {
        let value_string = value.into();

        //
        //
        if self.choices.contains(&value_string) {
            self.inner
                .lock()
                .await
                .set(StringCodec {
                    value: value_string,
                })
                .await?;
            Ok(())
        } else {
            Err(Error::EnumOutOfChoices(format!(
                "{:?} is not in {:?}",
                &value_string, self.choices
            )))
        }
    }
}
