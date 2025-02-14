use crate::{log_debug, Container, Error};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait]
/// Trait for driver that can read *IDN?
///
pub trait IdnReader: Sync + Send {
    /// Send a command and return the response
    ///
    async fn read_idn(&mut self) -> Result<String, Error>;
}

/// Mount the identity attribute in parent container
///
pub async fn mount<C: Container, I: IdnReader>(
    mut parent: C,
    interface: Arc<Mutex<I>>,
) -> Result<(), Error> {
    //
    // Create attribute
    let att_identity = parent
        .create_attribute("identity")
        .with_ro()
        .with_info("Identity string of the device")
        .finish_as_string()
        .await?;

    //
    // Create the local logger
    let logger = att_identity.logger();
    log_debug!(logger, "Mounting...");

    //
    // Just init
    let idn = interface.lock().await.read_idn().await?;
    log_debug!(logger, "IDN ({:?})", &idn);
    att_identity.set(idn).await?;

    //
    // End
    log_debug!(logger, "Mounting => OK");
    Ok(())
}
