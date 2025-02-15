use super::Instance;
use crate::Error;
use async_trait::async_trait;
use panduza::pubsub::PubSubOperator;

/// Actions of instances (specific for each driver)
///
#[async_trait]
pub trait Actions<O: PubSubOperator>: Send + Sync {
    /// Mount instance
    ///
    async fn mount(&mut self, mut instance: Instance<O>) -> Result<(), Error>;

    /// If this instance crashed, got an error or is not available anymore
    /// This function must monitor reboot condition and await them
    /// Once this function return, the instance will reboot
    ///
    async fn wait_reboot_event(&mut self, mut instance: Instance<O>);
}
