use crate::model::BooleanAccessorModel;
use crate::Error;
use crate::{log_debug, log_debug_mount_end, log_debug_mount_start, Container};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Mount the identity attribute in parent container
///
pub async fn mount<
    C: Container,
    I: BooleanAccessorModel + 'static,
    N: Into<String>,
    F: Into<String>,
>(
    mut parent: C,
    interface: Arc<Mutex<I>>,
    index: usize,
    name: N,
    info: F,
) -> Result<(), Error> {
    //
    // Create attribute
    let mut att = parent
        .create_attribute(name)
        .with_rw()
        .with_info(info)
        .start_as_boolean()
        .await?;

    //
    // Create the local logger
    let logger = att.logger().clone();
    log_debug_mount_start!(logger);

    //
    // Just init
    let value = interface.lock().await.get_boolean_at(index).await?;
    log_debug!(logger, "Initial value ({:?})", &value);
    att.set(value).await?;

    tokio::spawn(async move {
        loop {
            att.wait_for_commands().await;
            while let Some(command) = att.pop().await {
                //
                // Log
                log_debug!(att.logger(), "command received '{:?}'", command);

                //
                //
                interface
                    .lock()
                    .await
                    .set_boolean_at(index, command)
                    .await
                    .unwrap();

                //
                // Read back
                let read_back_value = interface.lock().await.get_boolean_at(index).await.unwrap();
                att.set(read_back_value).await.unwrap();
            }
        }
    });

    //
    // End
    log_debug_mount_end!(logger);
    Ok(())
}
