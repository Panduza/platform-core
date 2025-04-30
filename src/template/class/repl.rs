use crate::{
    log_debug, log_debug_mount_end, log_debug_mount_start, protocol::BytesDialogProtocol,
    Container, Error, Logger,
};
use async_trait::async_trait;
use bytes::Bytes;
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
pub async fn mount<A: Into<String>, C: Container, I: BytesDialogProtocol + 'static>(
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
        .start_as_string()
        .await?;

    let att_response = class_repl
        .create_attribute("response")
        .with_ro()
        .start_as_string()
        .await?;

    //
    // Execute action on each command received
    let logger_2 = att_command.logger().clone();
    let mut att_command_2 = att_command.clone();
    let att_response_2 = att_response.clone();
    tokio::spawn(async move {
        loop {
            while let Some(command) = att_command_2.pop().await {
                log_debug!(logger_2, "Command received {:?}", command);

                let response_result = connector.lock().await.ask(Bytes::from(command)).await;

                match response_result {
                    Ok(response) => {
                        let response_string = String::from_utf8_lossy(&response).to_string();
                        if let Err(e) = att_response_2.set(response_string).await {
                            log_debug!(logger_2, "Erreur lors de l'envoi de la réponse : {:?}", e);
                        }
                    }
                    Err(e) => {
                        log_debug!(logger_2, "Erreur dans ask() : {:?}", e);
                        let _ = att_response_2.set(format!("Erreur : {}", e)).await;
                    }
                }
            }
        }
    });
    /*spawn_on_command!(
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
    */
    //
    // End
    log_debug_mount_end!(logger);
    Ok(())
}
/*
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
*/
