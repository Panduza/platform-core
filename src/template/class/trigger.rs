use crate::{
    log_debug, log_debug_mount_end, log_debug_mount_start, log_trace, model::BooleanAccessorModel,
    Container, Error,
};
use async_trait::async_trait;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{Mutex, Notify},
    time::sleep,
};

#[async_trait]
///
///
pub trait Triggerable: Sync + Send {
    ///
    ///
    async fn on_trigger(&mut self) -> Result<(), Error>;
}

///
///
pub async fn mount<C: Container, I: Triggerable + Clone + 'static>(
    mut parent: C,
    mut interface: I,
) -> Result<(), Error> {
    //
    //
    let mut class_trigger = parent
        .create_class("trigger")
        .with_tag("trigger")
        .finish()
        .await;
    let logger = class_trigger.logger().clone();
    log_debug_mount_start!(logger);

    //
    //
    let mut att_single = class_trigger
        .create_attribute("single")
        .with_wo()
        .start_as_boolean()
        .await?;

    //
    // Execute action on each command received
    tokio::spawn(async move {
        loop {
            att_single.wait_for_commands().await;
            while let Some(command) = att_single.pop().await {
                interface.on_trigger().await.unwrap();
            }
        }
    });

    //
    //
    let att_cyclic = class_trigger
        .create_attribute("cyclic")
        .with_rw()
        .start_as_si("Hz", 0.0, 300.0, 2)
        .await?;

    //
    //
    let cycle_changed = Arc::new(Notify::new());
    let cycle_freq = Arc::new(Mutex::new(0.0));
    let mut cycle_step = Duration::from_secs(0xFFFFFFFF);

    //
    //

    // let cycle_freq_3 = cycle_freq.clone();
    // let att_cyclic_logger = att_cyclic.logger().clone();
    // let att_cyclic_2 = att_cyclic.clone();
    // let cycle_changed_2 = cycle_changed.clone();

    // spawn_on_command!(
    //     "on_command => trigger/cyclic",
    //     parent,
    //     att_cyclic,
    //     on_cyclic_command(
    //         att_cyclic_2.clone(),
    //         cycle_freq_3.clone(),
    //         cycle_changed_2.clone()
    //     )
    // );

    // //
    // //
    // let cycle_freq_2 = cycle_freq.clone();
    // let triggered_2 = triggered.clone();
    // spawn_loop!("loop => trigger ", parent, {
    //     tokio::select! {
    //         _ = cycle_changed.notified() => {
    //             let freq = *cycle_freq_2.lock().await;
    //             cycle_step = if freq > 0.0 {
    //                 Duration::from_secs_f64(1.0 / freq as f64)
    //             } else {
    //                 Duration::from_secs(0xFFFFFFFF)
    //             };
    //             log_debug!(att_cyclic_logger, "cycle changed {:?}Hz => {:?}", freq, cycle_step);
    //         }
    //         _ = sleep(cycle_step) => {
    //             log_trace!(att_cyclic_logger, "auto trig !");
    //             triggered_2.lock().await.on_trigger().await?;
    //         }
    //     }
    // });

    //
    //
    log_debug_mount_end!(logger);
    Ok(())
}

// /// On command callback
// ///
// async fn on_single_command<T: Triggerable + 'static>(
//     mut att_single: BooleanAttributeServer,
//     triggered: Arc<Mutex<T>>,
// ) -> Result<(), Error> {
//     while let Some(command) = att_single.pop_cmd().await {
//         //
//         // Log
//         log_trace!(att_single.logger(), "command received '{:?}'", command);

//         //
//         // action
//         triggered.lock().await.on_trigger().await?;
//     }
//     Ok(())
// }

// /// On command callback
// ///
// async fn on_cyclic_command(
//     mut att_cyclic: SiAttServer,
//     cycle_freq: Arc<Mutex<f32>>,
//     cycle_freq_changed: Arc<Notify>,
// ) -> Result<(), Error> {
//     while let Some(command) = att_cyclic.pop_cmd_as_f32().await {
//         //
//         // Log
//         log_debug!(att_cyclic.logger(), "command received '{:?}'", command);

//         match command {
//             Ok(c) => {
//                 //
//                 // action
//                 *cycle_freq.lock().await = c;
//                 att_cyclic.set_from_f32(c).await?;
//                 cycle_freq_changed.notify_waiters();
//             }
//             Err(e) => return Err(e),
//         }
//     }
//     Ok(())
// }
