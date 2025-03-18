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
    //
    let rst = parent.reset_signal();
    let logger2 = logger.clone();
    let att2 = att.clone();
    let mut interface2 = interface.clone();
    tokio::spawn(async move {
        loop {
            //
            // Just init
            let value = interface2.get_string_at(index).await.unwrap();
            log_debug!(logger2, "Initial value ({:?})", &value);
            att2.set(value).await.unwrap();

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
