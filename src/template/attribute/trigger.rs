use crate::model::TriggerAccessorModel;
use crate::Error;
use crate::{log_debug, log_debug_mount_end, log_debug_mount_start, Container};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Mount the identity attribute in parent container
///
pub async fn mount<
    C: Container,
    I: TriggerAccessorModel + 'static,
    N: Into<String>,
    F: Into<String>,
>(
    mut parent: C,
    mut interface: I,
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
        .start_as_trigger()
        .await?;

    //
    // Create the local logger
    let logger = att.logger().clone();
    log_debug_mount_start!(logger);

    //
    // Just init
    // let value = interface.get_number_at(index).await?;
    // log_debug!(logger, "Initial value ({:?})", &value);
    // att.set(0.0).await?;

    // //
    // let att_2 = att.clone();
    // spawn_on_command!(
    //     "on_command => boolean",
    //     parent,
    //     att_2,
    //     on_command(att_2.clone(), interface.clone(), index)
    // );

    tokio::spawn(async move {
        loop {
            att.wait_for_commands().await;
            while let Some(command) = att.pop().await {
                //
                // Log
                log_debug!(att.logger(), "command received '{:?}'", command);

                // //
                // //
                // interface.set_number_at(index, command).await.unwrap();

                // //
                // // Read back
                // let read_back_value = interface.get_boolean_at(index).await.unwrap();
                // att.set(read_back_value).await.unwrap();
            }
        }
    });

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
