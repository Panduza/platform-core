use super::Instance;
use crate::Error;
use async_trait::async_trait;

/// Actions of instances (specific for each driver)
///
#[async_trait]
pub trait Actions: Send + Sync {
    /// Mount driver instance and give him its structure
    ///
    async fn mount(&mut self, mut instance: Instance) -> Result<(), Error>;

    /// This instance crashed, got an error or is not available anymore
    /// This function must monitor reboot condition and await them
    /// Once this function return, the instance will reboot
    ///
    async fn wait_reboot_event(&mut self, mut instance: Instance);
}
