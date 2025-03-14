use crate::model::TriggerAccessorModel;
use crate::template::class::trigger::{self, Triggerable};
use crate::Error;
use crate::{log_debug, log_debug_mount_end, log_debug_mount_start, Container};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
struct TriggerWrap<I: TriggerAccessorModel> {
    interface: I,
    index: usize,
}

#[async_trait]
impl<I: TriggerAccessorModel> Triggerable for TriggerWrap<I> {
    async fn on_trigger(&mut self) -> Result<(), Error> {
        self.interface.trigger_at(self.index).await?;
        Ok(())
    }
}

/// Mount the identity attribute in parent container
///
pub async fn mount<
    C: Container,
    I: TriggerAccessorModel + Clone + 'static,
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
    // Do it like this waiting for better trigger attribute
    let trigger_wrap = TriggerWrap {
        // channel: 0,
        // att: att_data.clone(),
        interface: interface.clone(),
        index: index,
    };
    trigger::mount(parent, trigger_wrap).await?;

    // //
    // // Create attribute
    // let mut att = parent
    //     .create_attribute(name)
    //     .with_rw()
    //     .with_info(info)
    //     .start_as_trigger()
    //     .await?;

    // //
    // // Create the local logger
    // let logger = att.logger().clone();
    // log_debug_mount_start!(logger);

    // //
    // // Just init
    // // let value = interface.get_number_at(index).await?;
    // // log_debug!(logger, "Initial value ({:?})", &value);
    // // att.set(0.0).await?;

    // // //
    // // let att_2 = att.clone();
    // // spawn_on_command!(
    // //     "on_command => boolean",
    // //     parent,
    // //     att_2,
    // //     on_command(att_2.clone(), interface.clone(), index)
    // // );

    // tokio::spawn(async move {
    //     loop {
    //         att.wait_for_commands().await;
    //         while let Some(command) = att.pop().await {
    //             //
    //             // Log
    //             log_debug!(att.logger(), "command received '{:?}'", command);

    //             // //
    //             // //
    //             // interface.set_number_at(index, command).await.unwrap();

    //             // //
    //             // // Read back
    //             // let read_back_value = interface.get_boolean_at(index).await.unwrap();
    //             // att.set(read_back_value).await.unwrap();
    //         }
    //     }
    // });

    //
    // End
    // log_debug_mount_end!(logger);
    Ok(())
}
