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
pub async fn mount<C: Container, I: Triggerable + Clone + 'static, N: Into<String>>(
    mut parent: C,
    mut interface: I,
    name: N,
) -> Result<(), Error> {
    //
    //
    let mut class_trigger = parent
        .create_class(name.into())
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
    let mut interface_2 = interface.clone();
    tokio::spawn(async move {
        loop {
            att_single.wait_for_commands().await;
            while let Some(command) = att_single.pop().await {
                interface_2.on_trigger().await.unwrap();
            }
        }
    });

    //
    //
    let mut att_cyclic = class_trigger
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
    let cycle_freq_2 = cycle_freq.clone();
    let att_cyclic_logger = att_cyclic.logger().clone();
    let mut att_cyclic_2 = att_cyclic.clone();
    let cycle_changed_2 = cycle_changed.clone();
    tokio::spawn(async move {
        loop {
            att_cyclic_2.wait_for_commands().await;
            while let Some(command) = att_cyclic_2.pop().await {
                *cycle_freq_2.lock().await = command.try_into_f32().unwrap();
                att_cyclic_2.set(command).await.unwrap();
                cycle_changed_2.notify_waiters();
            }
        }
    });

    //
    //
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = cycle_changed.notified() => {
                    let freq = *cycle_freq.lock().await;
                    cycle_step = if freq > 0.0 {
                        Duration::from_secs_f64(1.0 / freq as f64)
                    } else {
                        Duration::from_secs(0xFFFFFFFF)
                    };
                    log_debug!(att_cyclic_logger, "cycle changed {:?}Hz => {:?}", freq, cycle_step);
                }
                _ = sleep(cycle_step) => {
                    log_trace!(att_cyclic_logger, "auto trig !");
                    interface.on_trigger().await.unwrap();
                }
            }
        }
    });

    //
    //
    log_debug_mount_end!(logger);
    Ok(())
}
