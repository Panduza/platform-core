use crate::{instance::attribute_builder::AttributeServerBuilder, Logger};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default, Debug)]
struct BooleanDataPack {
    /// Last value received
    ///
    last: Option<bool>,

    /// Queue of value (need to be poped)
    ///
    queue: Vec<bool>,
}

///
///
#[derive(Clone)]
pub struct BooleanAttributeServer {
    /// Local logger
    ///
    logger: Logger,

    /// Inner server implementation
    ///
    pack: Arc<Mutex<BooleanDataPack>>,
}

impl BooleanAttributeServer {
    // /// Clone as an element object
    // ///
    // pub fn clone_as_element(&self) -> Element {
    //     Element::AsBoolean(self.clone())
    // }

    ///
    ///
    pub fn r#type() -> String {
        "boolean".to_string()
    }

    ///
    ///
    pub fn new(builder: AttributeServerBuilder) -> Self {
        let topic = builder.topic.as_ref().unwrap().clone();
        Self {
            logger: Logger::new_for_attribute_from_topic(topic.clone()),
            pack: Arc::new(Mutex::new(BooleanDataPack::default())),
        }
    }

    // ///
    // /// Get the value of the attribute
    // /// If None, the first value is not yet received
    // ///
    // pub async fn pop_cmd(&mut self) -> Option<bool> {
    //     self.inner
    //         .lock()
    //         .await
    //         .pop_cmd()
    //         .and_then(|v| Some(v.value))
    // }

    // ///
    // /// Get the value of the attribute
    // /// If None, the first value is not yet received
    // ///
    // pub async fn get_last_cmd(&self) -> Option<bool> {
    //     return self
    //         .inner
    //         .lock()
    //         .await
    //         .get_last_cmd()
    //         .and_then(|v| Some(v.value));
    // }

    // /// Set the value of the attribute
    // ///
    // pub async fn set(&self, value: bool) -> Result<(), Error> {
    //     self.inner
    //         .lock()
    //         .await
    //         .set(BooleanCodec { value: value })
    //         .await?;
    //     Ok(())
    // }
}
