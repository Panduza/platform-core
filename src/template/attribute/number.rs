use crate::model::NumberAccessorModel;
use crate::Error;
use crate::{log_debug, log_debug_mount_end, log_debug_mount_start, Container};
use async_trait::async_trait;
use panduza::fbs::number::NumberBuffer;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Mount the identity attribute in parent container
///
pub async fn mount<
    C: Container,
    I: NumberAccessorModel + 'static,
    N: Into<String>,
    F: Into<String>,
    U: Into<String>,
>(
    mut parent: C,
    mut interface: I,
    index: usize,
    name: N,
    info: F,
    unit: U,
    min: f64,
    max: f64,
    decimals: usize,
) -> Result<(), Error> {
    //
    // Create attribute
    let mut att = parent
        .create_attribute(name)
        .with_rw()
        .with_info(info)
        .start_as_si(unit, min, max, decimals)
        .await?;

    //
    // Create the local logger
    let logger = att.logger().clone();
    log_debug_mount_start!(logger);

    //
    // Just init
    let value = interface.get_number_at(index).await?;
    log_debug!(logger, "Initial value ({:?})", &value);
    att.set(NumberBuffer::from_float_with_decimals(value, decimals))
        .await?;

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
                    .set_number_at(index, command.try_into_f32().unwrap())
                    .await
                    .unwrap();

                //
                // Read back
                let read_back_value = interface.get_number_at(index).await.unwrap();
                att.set(NumberBuffer::from_float_with_decimals(
                    read_back_value,
                    decimals,
                ))
                .await
                .unwrap();
            }
        }
    });
    //
    // End
    log_debug_mount_end!(logger);
    Ok(())
}
