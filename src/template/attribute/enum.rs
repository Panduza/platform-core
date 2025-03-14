use crate::model::StringAccessorModel;
use crate::Error;
use crate::{log_debug, log_debug_mount_end, log_debug_mount_start, Container};

/// Mount the identity attribute in parent container
///
pub async fn mount<
    C: Container,
    I: StringAccessorModel + Clone + 'static,
    N: Into<String>,
    F: Into<String>,
    S: Into<String>,
>(
    mut parent: C,
    mut interface: I,
    index: usize,
    name: N,
    info: F,
    choices: Vec<S>,
) -> Result<(), Error> {
    //
    // Create attribute
    let mut att = parent
        .create_attribute(name)
        .with_rw()
        .with_info(info)
        .start_as_enum(choices)
        .await?;

    //
    // Create the local logger
    let logger = att.logger().clone();
    log_debug_mount_start!(logger);

    //
    // Just init
    let value = interface.get_string_at(index).await?;
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
                interface.set_string_at(index, command).await.unwrap();

                //
                // Read back
                let read_back_value = interface.get_string_at(index).await.unwrap();
                att.set(read_back_value).await.unwrap();
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
// async fn on_command<I: StringAccessorModel + 'static>(
//     mut att: EnumAttServer,
//     interface: Arc<Mutex<I>>,
//     index: usize,
// ) -> Result<(), Error> {
//     while let Some(command) = att.pop_cmd().await {
//         //
//         // Log
//         log_debug!(att.logger(), "command received '{:?}'", command);

//         match command {
//             Ok(v) => {
//                 //
//                 //
//                 interface.lock().await.set_string_at(index, &v).await?;

//                 //
//                 // Read back
//                 let read_back_value = interface.lock().await.get_string_at(index).await?;
//                 att.set(read_back_value).await?;
//             }
//             Err(_) => {}
//         }
//     }
//     Ok(())
// }
