use crate::{log_debug, log_debug_mount_end, log_debug_mount_start, spawn_on_command, Container};
use crate::{EnumAttServer, Error};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait]
///
///
pub trait StringAccessorModel: Sync + Send {
    ///
    ///
    async fn get_string_at(&mut self, index: usize) -> Result<String, Error>;
    ///
    ///
    async fn set_string_at(&mut self, index: usize, value: &String) -> Result<(), Error>;
}

/// Mount the identity attribute in parent container
///
pub async fn mount<
    C: Container,
    I: StringAccessorModel + 'static,
    N: Into<String>,
    F: Into<String>,
    S: Into<String>,
>(
    mut parent: C,
    interface: Arc<Mutex<I>>,
    index: usize,
    name: N,
    info: F,
    choices: Vec<S>,
) -> Result<(), Error> {
    //
    // Create attribute
    let att_boolean_rw = parent
        .create_attribute(name)
        .with_rw()
        .with_info(info)
        .finish_as_enum(choices)
        .await?;

    //
    // Create the local logger
    let logger = att_boolean_rw.logger();
    log_debug_mount_start!(logger);

    //
    // Just init
    let value = interface.lock().await.get_string_at(index).await?;
    log_debug!(logger, "Initial value ({:?})", &value);
    att_boolean_rw.set(value).await?;

    //
    let att_boolean_rw_2 = att_boolean_rw.clone();
    spawn_on_command!(
        "on_command => boolean",
        parent,
        att_boolean_rw_2,
        on_command(att_boolean_rw_2.clone(), interface.clone(), index)
    );

    //
    // End
    log_debug_mount_end!(logger);
    Ok(())
}

///
///
async fn on_command<I: StringAccessorModel + 'static>(
    mut att: EnumAttServer,
    interface: Arc<Mutex<I>>,
    index: usize,
) -> Result<(), Error> {
    while let Some(command) = att.pop_cmd().await {
        //
        // Log
        log_debug!(att.logger(), "command received '{:?}'", command);

        match command {
            Ok(v) => {
                //
                //
                interface.lock().await.set_string_at(index, &v).await?;

                //
                // Read back
                let read_back_value = interface.lock().await.get_string_at(index).await?;
                att.set(read_back_value).await?;
            }
            Err(_) => {}
        }
    }
    Ok(())
}
