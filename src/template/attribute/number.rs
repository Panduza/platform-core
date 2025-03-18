use crate::model::NumberAccessorModel;
use crate::{interface, Error};
use crate::{log_debug, log_debug_mount_end, log_debug_mount_start, Container};
use async_trait::async_trait;
use panduza::fbs::number::NumberBuffer;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Mount the identity attribute in parent container
///
pub async fn mount<
    C: Container,
    I: NumberAccessorModel + Clone + 'static,
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
    let rst = parent.reset_signal();
    let logger2 = logger.clone();
    let att2 = att.clone();
    let mut interface2 = interface.clone();
    tokio::spawn(async move {
        loop {
            //
            // Just init
            let value = interface2.get_number_at(index).await.unwrap();
            log_debug!(logger2, "Initial value ({:?})", &value);
            att2.set(NumberBuffer::from_float_with_decimals(value, decimals))
                .await
                .unwrap();

            // Then wait for next reset
            rst.notified().await;
        }
    });

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
