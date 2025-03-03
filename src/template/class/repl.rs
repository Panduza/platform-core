use crate::{
    log_debug, log_debug_mount_end, log_debug_mount_start, spawn_on_command, Container, Error,
    Logger, StringAttServer,
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait]
/// Protocol in which we send a text command and the device respond with another text
///
pub trait ReplProtocol: Sync + Send {
    /// Evaluate the command and return the response
    ///
    async fn eval(&mut self, command: String) -> Result<String, Error>;
}

/// Mount the identity attribute
///
pub async fn mount<A: Into<String>, C: Container, I: ReplProtocol + 'static>(
    name: A,
    mut parent: C,
    connector: Arc<Mutex<I>>,
) -> Result<(), Error> {
    //
    //
    let mut class_repl = parent
        .create_class(&name.into())
        .with_tag("REPL")
        .finish()
        .await;
    let logger = class_repl.logger().clone();
    log_debug_mount_start!(logger);

    let att_command = class_repl
        .create_attribute("command")
        .with_wo()
        .finish_as_string()
        .await?;

    let att_response = class_repl
        .create_attribute("response")
        .with_ro()
        .finish_as_string()
        .await?;

    //
    // Execute action on each command received
    let logger_2 = att_command.logger().clone();
    let att_command_2 = att_command.clone();
    let att_response_2 = att_response.clone();
    spawn_on_command!(
        "on_command => relp",
        parent,
        att_command_2,
        on_command(
            logger_2.clone(),
            att_command_2.clone(),
            att_response_2.clone(),
            connector.clone()
        )
    );

    //
    // End
    log_debug_mount_end!(logger);
    Ok(())
}

/// On command callback
///
async fn on_command<I: ReplProtocol + 'static>(
    logger: Logger,
    mut att_command: StringAttServer,
    att_response: StringAttServer,
    connector: Arc<Mutex<I>>,
) -> Result<(), Error> {
    while let Some(command) = att_command.pop_cmd().await {
        //
        // Log
        log_debug!(logger, "Command received {:?}", command);
        let response = connector.lock().await.eval(command).await?;
        att_response.set(response).await?;
    }
    Ok(())
}
