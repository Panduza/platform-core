use crate::model::NumberAccessorModel;
use crate::Error;
use crate::{log_debug, log_debug_mount_end, log_debug_mount_start, Container};
use async_trait::async_trait;
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
    let att_number_rw = parent
        .create_attribute(name)
        .with_rw()
        .with_info(info)
        .start_as_si(unit, min, max, decimals)
        .await?;

    //
    // Create the local logger
    let logger = att_number_rw.logger();
    log_debug_mount_start!(logger);

    //
    // Just init
    let value = interface.get_number_at(index).await?;
    log_debug!(logger, "Initial value ({:?})", &value);
    // att_number_rw.set(&value).await?;

    // //
    // let att_number_rw_2 = att_number_rw.clone();
    // spawn_on_command!(
    //     "on_command => boolean",
    //     parent,
    //     att_number_rw_2,
    //     on_command(att_number_rw_2.clone(), interface.clone(), index)
    // );

    //
    // End
    log_debug_mount_end!(logger);
    Ok(())
}

// ///
// ///
// async fn on_command<I: NumberAccessorModel + 'static>(
//     mut att: SiAttServer,
//     interface: Arc<Mutex<I>>,
//     index: usize,
// ) -> Result<(), Error> {
//     while let Some(command) = att.pop_cmd().await {
//         //
//         // Log
//         log_debug!(att.logger(), "command received '{:?}'", command);

//         //
//         //
//         interface
//             .lock()
//             .await
//             .set_number_at(index, &command)
//             .await?;

//         //
//         // Read back
//         let read_back_value = interface.get_number_at(index).await?;
//         att.set(&read_back_value).await?;
//     }
//     Ok(())
// }
